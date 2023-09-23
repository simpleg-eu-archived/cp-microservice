use std::collections::HashMap;
use std::fmt::Debug;
use std::mem;
use std::mem::Discriminant;
use std::sync::Arc;
use std::time::Duration;

use async_channel::{unbounded, Receiver};
use log::info;
use tokio::time::timeout;

use crate::error::Error;
use crate::storage::executor::Executor;

pub struct Dispatch<StorageRequestType: Debug> {
    receiver: Receiver<StorageRequestType>,
    executors: HashMap<Discriminant<StorageRequestType>, Executor<(), StorageRequestType>>,
}

impl<StorageRequestType: Debug> Dispatch<StorageRequestType> {
    pub fn new(
        receiver: Receiver<StorageRequestType>,
        executors: HashMap<Discriminant<StorageRequestType>, Executor<(), StorageRequestType>>,
    ) -> Dispatch<StorageRequestType> {
        Dispatch {
            receiver,
            executors,
        }
    }

    pub async fn run(self) {
        loop {
            let storage_request = match self.receiver.recv().await {
                Ok(storage_request) => storage_request,
                Err(_) => {
                    info!("failed to receive storage request");
                    continue;
                }
            };

            let executor = match self.executors.get(&mem::discriminant(&storage_request)) {
                Some(executor) => executor,
                None => {
                    info!(
                        "failed to find discriminant for storage request: {:?}",
                        storage_request
                    );
                    continue;
                }
            };

            if let Err(error) = executor((), storage_request).await {
                info!("storage executor returned error: {}", error);
            }
        }
    }
}

#[cfg(test)]
async fn dummy_executor(storage_request: StorageRequest) -> Result<(), Error> {
    match storage_request {
        StorageRequest::DummyElement(message, replier) => {
            replier.send(message).expect("failed to send reply");
            return Ok(());
        }
        _ => panic!("unexpected execution path"),
    }
}

#[derive(Debug)]
pub enum StorageRequest {
    DummyElement(String, tokio::sync::oneshot::Sender<String>),
}

#[tokio::test]
pub async fn run_expected_executor() {
    const TEST_MESSAGE: &str = "test";
    let (dummy_sender, _) = tokio::sync::oneshot::channel::<String>();
    let executor: Executor<(), StorageRequest> =
        Arc::new(|storage_connection, storage_request| Box::pin(dummy_executor(storage_request)));
    let executors: HashMap<Discriminant<StorageRequest>, Executor<(), StorageRequest>> =
        HashMap::from([(
            mem::discriminant(&StorageRequest::DummyElement("".to_string(), dummy_sender)),
            executor,
        )]);
    let (sender, receiver) = unbounded::<StorageRequest>();

    let dispatch: Dispatch<StorageRequest> = Dispatch::new(receiver, executors);

    tokio::spawn(dispatch.run());

    let (reply_sender, reply_receiver) = tokio::sync::oneshot::channel::<String>();

    let request: StorageRequest =
        StorageRequest::DummyElement(TEST_MESSAGE.to_string(), reply_sender);
    sender
        .send(request)
        .await
        .expect("failed to send storage request");

    let reply = timeout(Duration::from_millis(200u64), reply_receiver)
        .await
        .expect("timeout waiting for reply")
        .expect("failed to receive reply");

    assert_eq!(TEST_MESSAGE, reply);
}
