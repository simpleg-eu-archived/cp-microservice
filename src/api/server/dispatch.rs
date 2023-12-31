use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::api::server::input::action::Action;
use crate::api::server::input::executor::Executor;
use crate::api::server::input::input::Input;
use crate::api::server::input::input_data::InputData;
use crate::api::server::input::input_plugin::InputPlugin;
use crate::api::server::input::replier::Replier;
use crate::api::shared::request::Request;
use crate::api::shared::request_header::RequestHeader;
use async_channel::Sender;
use async_trait::async_trait;
use log::{info, warn};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};
use tokio_util::sync::CancellationToken;

use crate::core::error::Error;

pub struct Dispatch<InputImpl: 'static + Input + Send, LogicRequestType: 'static + Send> {
    inputs: Vec<InputImpl>,
    actions: Arc<HashMap<String, Action<LogicRequestType>>>,
    sender: Sender<LogicRequestType>,
    plugins: Arc<Vec<Arc<dyn InputPlugin + Send + Sync>>>,
}

impl<InputImpl: 'static + Input + Send, LogicRequestType: 'static + Send>
    Dispatch<InputImpl, LogicRequestType>
{
    pub fn new(
        inputs: Vec<InputImpl>,
        actions: HashMap<String, Action<LogicRequestType>>,
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

    pub async fn run(self, cancellation_token: CancellationToken) -> Vec<JoinHandle<()>> {
        let mut api_handles = Vec::new();

        for input in self.inputs {
            let actions_pointer: Arc<HashMap<String, Action<LogicRequestType>>> =
                self.actions.clone();
            let logic_request_sender = self.sender.clone();
            let plugins_pointer = self.plugins.clone();

            api_handles.push(tokio::spawn(run_dispatch_input(
                input,
                actions_pointer,
                logic_request_sender,
                plugins_pointer,
                cancellation_token.clone(),
            )));
        }

        api_handles
    }
}

fn get_filtered_out_plugins_for_action<LogicRequestType>(
    action: &str,
    actions: &Arc<HashMap<String, Action<LogicRequestType>>>,
) -> Vec<String> {
    match actions.get(action) {
        Some(action) => action.filter_out_plugins(),
        None => Vec::new(),
    }
}

async fn handle_input_data<LogicRequestType: 'static + Send>(
    input_data: InputData,
    actions: &Arc<HashMap<String, Action<LogicRequestType>>>,
    sender: Sender<LogicRequestType>,
) {
    let action = input_data.request.header().action();

    match actions.get(action) {
        Some(action) => {
            let executor = action.executor();
            let action_result = executor(input_data.request, sender).await;

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

async fn run_dispatch_input<InputImpl: 'static + Input + Send, LogicRequestType: 'static + Send>(
    mut input: InputImpl,
    actions_pointer: Arc<HashMap<String, Action<LogicRequestType>>>,
    logic_request_sender: Sender<LogicRequestType>,
    plugins_pointer: Arc<Vec<Arc<dyn InputPlugin + Send + Sync>>>,
    cancellation_token: CancellationToken,
) {
    loop {
        if cancellation_token.is_cancelled() {
            info!("cancellation token is cancelled, api dispatch is stopping");

            break;
        }

        let result = input.receive().await;

        match result {
            Ok(mut input_data) => {
                if plugins_pointer.len() == 0 {
                    handle_input_data::<LogicRequestType>(
                        input_data,
                        &actions_pointer,
                        logic_request_sender.clone(),
                    )
                    .await;
                } else {
                    let filtered_out_plugins =
                        get_filtered_out_plugins_for_action::<LogicRequestType>(
                            input_data.request.header().action(),
                            &actions_pointer,
                        );

                    for (index, plugin) in plugins_pointer.as_slice().iter().enumerate() {
                        if !filtered_out_plugins.contains(&plugin.id().to_string()) {
                            input_data = match plugin.handle_input_data(input_data).await {
                                Ok(input_data) => input_data,
                                Err((input_data, error)) => {
                                    let replier = input_data.replier;

                                    let error_value = match serde_json::to_value(error.clone()) {
                                        Ok(error_value) => error_value,
                                        Err(error) => {
                                            json!(format!("failed to process request: {}", error))
                                        }
                                    };

                                    match replier(error_value).await {
                                        Ok(_) => (),
                                        Err(error) => {
                                            warn!("failed to reply when plugin failed: {}", error)
                                        }
                                    }

                                    warn!("plugin failed to handle input data: {}", error);
                                    break;
                                }
                            };
                        }

                        if index == plugins_pointer.len() - 1 {
                            handle_input_data::<LogicRequestType>(
                                input_data,
                                &actions_pointer,
                                logic_request_sender.clone(),
                            )
                            .await;

                            break;
                        }
                    }
                }
            }
            Err(error) => {
                warn!("failed to receive input: {}", error);
            }
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

    tokio::spawn(dispatch.run(CancellationToken::new()));

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
    fn id(&self) -> &str {
        "dummy"
    }

    async fn handle_input_data(
        &self,
        input_data: InputData,
    ) -> Result<InputData, (InputData, Error)> {
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

    tokio::spawn(dispatch.run(CancellationToken::new()));

    let mut sum: u8 = 0;

    for _ in 0..2 {
        sum += timeout(Duration::from_millis(200u64), plugin_receiver.recv())
            .await
            .expect("timed out waiting for plugin to send byte")
            .expect("failed to receive byte from plugin");
    }

    assert_eq!(EXPECTED_SUM, sum);
}
