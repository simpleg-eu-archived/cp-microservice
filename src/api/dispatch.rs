use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_channel::Sender;
use async_trait::async_trait;
use log::{info, warn};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};

use crate::api::async_callback::AsyncCallback;
use crate::api::input::input::Input;
use crate::api::input::input_data::InputData;
use crate::api::input::input_plugin::InputPlugin;
use crate::api::input::replier::Replier;
use crate::api::input::request::Request;
use crate::api::input::request_header::RequestHeader;
use crate::error::Error;

pub struct Dispatch<InputImpl: 'static + Input + Send, LogicRequestType: 'static + Send> {
    inputs: Vec<InputImpl>,
    actions: Arc<HashMap<String, AsyncCallback<LogicRequestType>>>,
    sender: Sender<LogicRequestType>,
    plugins: Arc<Vec<Arc<dyn InputPlugin + Send + Sync>>>,
}

impl<InputImpl: 'static + Input + Send, LogicRequestType: 'static + Send>
    Dispatch<InputImpl, LogicRequestType>
{
    pub fn new(
        inputs: Vec<InputImpl>,
        actions: HashMap<String, AsyncCallback<LogicRequestType>>,
        sender: Sender<LogicRequestType>,
        plugins: Vec<Arc<dyn InputPlugin + Send + Sync>>,
    ) -> Dispatch<InputImpl, LogicRequestType> {
        Dispatch {
            inputs,
            actions: Arc::new(actions),
            sender,
            plugins: Arc::new(plugins),
        }
    }

    pub async fn run(self) {
        for mut input in self.inputs {
            let actions_pointer = self.actions.clone();
            let logic_request_sender = self.sender.clone();
            let plugins_pointer = self.plugins.clone();

            tokio::spawn(async move {
                loop {
                    let result = input.receive();
                    let sender = logic_request_sender.clone();

                    match result.await {
                        Ok(mut input_data) => {
                            for plugin in plugins_pointer.as_slice() {
                                input_data = plugin.handle_input_data(input_data).await.unwrap();
                            }

                            handle_input_data::<LogicRequestType>(
                                input_data,
                                &actions_pointer,
                                sender,
                            )
                            .await;
                        }
                        Err(error) => {
                            warn!("failed to receive input: {}", error);
                        }
                    }
                }
            });
        }
    }
}

async fn handle_input_data<LogicRequestType: 'static + Send>(
    input_data: InputData,
    actions: &Arc<HashMap<String, AsyncCallback<LogicRequestType>>>,
    sender: Sender<LogicRequestType>,
) {
    let action = input_data.request.header().action();

    match actions.get(action) {
        Some(action) => {
            let action_result = action(input_data.request, sender).await;

            let replier: Replier = input_data.replier;
            if let Err(error) = replier(json!(action_result)).await {
                warn!("failed to reply with action_result: {}", error);
            }
        }
        None => {
            info!("unknown action received: {}", action);
        }
    }
}

#[cfg(test)]
pub struct LogicRequest {}

pub struct InputTimedImpl {
    sleep_duration: Duration,
    sender: tokio::sync::mpsc::Sender<()>,
}

impl InputTimedImpl {
    pub fn new(sleep_duration: Duration, sender: tokio::sync::mpsc::Sender<()>) -> InputTimedImpl {
        InputTimedImpl {
            sleep_duration,
            sender,
        }
    }
}

#[async_trait]
impl Input for InputTimedImpl {
    async fn receive(&mut self) -> Result<InputData, Error> {
        sleep(self.sleep_duration).await;
        self.sender
            .send(())
            .await
            .expect("failed to send empty message");

        Ok(InputData {
            request: Request::new(
                RequestHeader::new("".to_string(), "".to_string()),
                Value::Null,
            ),
            replier: Arc::new(move |value: Value| Box::pin(async { Ok(()) })),
        })
    }
}

#[tokio::test]
pub async fn handle_multiple_inputs_concurrently() {
    let sleep_duration: Duration = Duration::from_millis(500u64);
    let max_execution_duration: Duration = Duration::from_millis(900u64);
    let expected_inputs: u8 = 2;

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<()>(1024usize);
    let (logic_request_sender, _) = async_channel::unbounded::<LogicRequest>();
    let inputs: Vec<InputTimedImpl> = vec![
        InputTimedImpl::new(sleep_duration, sender.clone()),
        InputTimedImpl::new(sleep_duration, sender.clone()),
    ];
    let dispatch: Dispatch<InputTimedImpl, LogicRequest> =
        Dispatch::new(inputs, HashMap::new(), logic_request_sender, vec![]);

    tokio::spawn(dispatch.run());

    timeout(max_execution_duration, async move {
        let mut count: u8 = 0;

        for _ in 0..expected_inputs {
            if (receiver.recv().await).is_some() {
                count += 1;
            }
        }

        assert_eq!(expected_inputs, count);
    })
    .await
    .expect("inputs are not being received concurrently");
}

pub struct InputDummyImpl {
    has_message_been_sent: RwLock<bool>,
}

impl Default for InputDummyImpl {
    fn default() -> Self {
        InputDummyImpl {
            has_message_been_sent: RwLock::new(false),
        }
    }
}

#[async_trait]
impl Input for InputDummyImpl {
    async fn receive(&mut self) -> Result<InputData, Error> {
        if *self.has_message_been_sent.try_read().unwrap() {
            loop {
                sleep(Duration::MAX).await;
            }
        }

        let request = Request::new(
            RequestHeader::new("".to_string(), "".to_string()),
            Value::Null,
        );
        let replier: Replier = Arc::new(move |value| Box::pin(async { Ok(()) }));

        if !(*self.has_message_been_sent.try_read().unwrap()) {
            *self.has_message_been_sent.try_write().unwrap() = true;
        }

        Ok(InputData { request, replier })
    }
}

pub struct DummyPlugin {
    send_value: u8,
    sender: tokio::sync::mpsc::Sender<u8>,
}

impl DummyPlugin {
    pub fn new(send_value: u8, sender: tokio::sync::mpsc::Sender<u8>) -> DummyPlugin {
        DummyPlugin { send_value, sender }
    }
}

#[async_trait]
impl InputPlugin for DummyPlugin {
    async fn handle_input_data(&self, input_data: InputData) -> Result<InputData, Error> {
        self.sender.send(self.send_value).await.unwrap();

        Ok(input_data)
    }
}

#[tokio::test]
pub async fn execute_specified_plugins_for_each_input() {
    const EXPECTED_SUM: u8 = 24u8;

    let inputs: Vec<InputDummyImpl> = vec![InputDummyImpl::default(), InputDummyImpl::default()];

    let (sender, _) = async_channel::unbounded();

    let (plugin_sender, mut plugin_receiver) = tokio::sync::mpsc::channel::<u8>(1024usize);

    let plugins: Vec<Arc<dyn InputPlugin + Send + Sync>> = vec![
        Arc::new(DummyPlugin::new(13u8, plugin_sender.clone())),
        Arc::new(DummyPlugin::new(11u8, plugin_sender)),
    ];

    let dispatch: Dispatch<InputDummyImpl, LogicRequest> =
        Dispatch::new(inputs, HashMap::new(), sender, plugins);

    tokio::spawn(dispatch.run());

    let mut sum: u8 = 0;

    for _ in 0..2 {
        sum += timeout(Duration::from_millis(200u64), plugin_receiver.recv())
            .await
            .expect("timed out waiting for plugin to send byte")
            .expect("failed to receive byte from plugin");
    }

    assert_eq!(EXPECTED_SUM, sum);
}
