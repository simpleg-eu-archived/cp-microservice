use std::collections::HashMap;
use std::fmt::Debug;
use std::mem;
use std::mem::Discriminant;
use std::sync::Arc;
use std::time::Duration;

use async_channel::{Receiver, Sender};
use log::info;
use tokio::time::timeout;

use crate::error::Error;
use crate::logic::executor::Executor;

pub struct Dispatch<LogicRequestType: Debug, StorageRequestType> {
    logic_request_receiver: Receiver<LogicRequestType>,
    executors:
        HashMap<Discriminant<LogicRequestType>, Executor<LogicRequestType, StorageRequestType>>,
    storage_request_sender: Sender<StorageRequestType>,
}

impl<LogicRequestType: Debug, StorageRequestType> Dispatch<LogicRequestType, StorageRequestType> {
    pub fn new(
        logic_request_receiver: Receiver<LogicRequestType>,
        executors: HashMap<
            Discriminant<LogicRequestType>,
            Executor<LogicRequestType, StorageRequestType>,
        >,
        storage_request_sender: Sender<StorageRequestType>,
    ) -> Dispatch<LogicRequestType, StorageRequestType> {
        Dispatch {
            logic_request_receiver,
            executors,
            storage_request_sender,
        }
    }

    pub async fn run(self) {
        loop {
            let logic_request = match self.logic_request_receiver.recv().await {
                Ok(logic_request) => logic_request,
                Err(_) => {
                    info!("failed to receive logic request");
                    continue;
                }
            };

            let executor = match self.executors.get(&mem::discriminant(&logic_request)) {
                Some(executor) => executor,
                None => {
                    info!(
                        "failed to find discriminant for logic request: {:?}",
                        logic_request
                    );
                    continue;
                }
            };

            if let Err(error) = executor(logic_request, self.storage_request_sender.clone()).await {
                info!("executor returned error: {}", error);
            }
        }
    }
}

#[cfg(test)]
const TEST_STORAGE_REQUEST_VALUE: &str = "ok";

async fn dummy_executor(
    value: LogicRequest,
    storage_sender: Sender<StorageRequest>,
) -> Result<(), Error> {
    storage_sender
        .send(StorageRequest::DummyElement(
            TEST_STORAGE_REQUEST_VALUE.to_string(),
        ))
        .await
        .expect("failed to send storage request");

    Ok(())
}

#[derive(Debug)]
pub enum LogicRequest {
    DummyElement(String),
}

pub enum StorageRequest {
    DummyElement(String),
}

#[tokio::test]
pub async fn run_expected_executors() {
    let exec: Executor<LogicRequest, StorageRequest> =
        Arc::new(|logic_request, storage_request_sender| {
            Box::pin(dummy_executor(logic_request, storage_request_sender))
        });

    let executors: HashMap<Discriminant<LogicRequest>, Executor<LogicRequest, StorageRequest>> =
        HashMap::from([(
            mem::discriminant(&LogicRequest::DummyElement("".to_string())),
            exec,
        )]);

    let (sender, receiver) = async_channel::unbounded::<LogicRequest>();
    let (storage_request_sender, storage_request_receiver) =
        async_channel::unbounded::<StorageRequest>();

    let dispatch: Dispatch<LogicRequest, StorageRequest> =
        Dispatch::new(receiver, executors, storage_request_sender);

    tokio::spawn(dispatch.run());

    sender
        .send(LogicRequest::DummyElement("random".to_string()))
        .await
        .expect("failed to send logic request");

    let request: StorageRequest = timeout(
        Duration::from_millis(200u64),
        storage_request_receiver.recv(),
    )
    .await
    .expect("timeout waiting for storage request")
    .expect("failed to receive storage request");

    match request {
        StorageRequest::DummyElement(value) => assert_eq!(TEST_STORAGE_REQUEST_VALUE, value),
        _ => panic!("unexpected storage request value received"),
    }
}
