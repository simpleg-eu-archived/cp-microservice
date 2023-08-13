use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_channel::Sender;
use async_trait::async_trait;
use log::{info, warn};
use serde_json::{json, Value};
use tokio::time::{sleep, timeout};

use crate::api::async_callback::AsyncCallback;
use crate::api::input::input::Input;
use crate::api::input::input_data::InputData;
use crate::api::input::replier::Replier;
use crate::api::input::request::Request;
use crate::api::input::request_header::RequestHeader;
use crate::error::Error;

pub struct Dispatch<InputImpl: 'static + Input + Send, LogicRequestType: 'static + Send> {
    inputs: Vec<InputImpl>,
    actions: Arc<HashMap<String, AsyncCallback<LogicRequestType>>>,
    sender: Sender<LogicRequestType>,
}

impl<InputImpl: 'static + Input + Send, LogicRequestType: 'static + Send>
    Dispatch<InputImpl, LogicRequestType>
{
    pub fn new(
        inputs: Vec<InputImpl>,
        actions: HashMap<String, AsyncCallback<LogicRequestType>>,
        sender: Sender<LogicRequestType>,
    ) -> Dispatch<InputImpl, LogicRequestType> {
        Dispatch {
            inputs,
            actions: Arc::new(actions),
            sender,
        }
    }

    pub async fn run(self) {
        for input in self.inputs {
            let actions_pointer = self.actions.clone();
            let logic_request_sender = self.sender.clone();

            tokio::spawn(async move {
                loop {
                    let result = input.receive();
                    let sender = logic_request_sender.clone();

                    match result.await {
                        Ok(input_data) => {
                            let action = input_data.request.header().action();

                            match actions_pointer.get(action) {
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
                        Err(error) => {
                            warn!("failed to receive input: {}", error);
                        }
                    }
                }
            });
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
    async fn receive(&self) -> Result<InputData, Error> {
        sleep(self.sleep_duration).await;
        self.sender
            .send(())
            .await
            .expect("failed to send empty message");

        Ok(InputData {
            request: Request::new(RequestHeader::new("".to_string()), Value::Null),
            replier: Arc::new(move |value: Value| Box::pin(async { Ok(()) })),
        })
    }
}

#[tokio::test]
pub async fn handle_multiple_inputs_concurrently() {
    let sleep_duration: Duration = Duration::from_millis(500u64);
    let max_execution_duration: Duration = Duration::from_millis(1500u64);
    let expected_inputs: u8 = 2;

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<()>(1024usize);
    let (logic_request_sender, _) = async_channel::unbounded::<LogicRequest>();
    let inputs: Vec<InputTimedImpl> = vec![
        InputTimedImpl::new(sleep_duration, sender.clone()),
        InputTimedImpl::new(sleep_duration, sender.clone()),
    ];
    let dispatch: Dispatch<InputTimedImpl, LogicRequest> =
        Dispatch::new(inputs, HashMap::new(), logic_request_sender);

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
