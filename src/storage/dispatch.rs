use std::{
    collections::HashMap,
    fmt::Debug,
    mem::{self, Discriminant},
};

use async_channel::Receiver;
use log::info;
use tokio_util::sync::CancellationToken;

use crate::storage::executor::Executor;

pub struct Dispatch<StorageRequestType: Debug> {
    storage_request_receiver: Receiver<StorageRequestType>,
    executors: HashMap<Discriminant<StorageRequestType>, Executor<StorageRequestType>>,
    cancellation_token: CancellationToken,
}

impl<StorageRequestType: Debug> Dispatch<StorageRequestType> {
    pub fn new(
        storage_request_receiver: Receiver<StorageRequestType>,
        executors: HashMap<Discriminant<StorageRequestType>, Executor<StorageRequestType>>,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            storage_request_receiver,
            executors,
            cancellation_token,
        }
    }

    pub async fn run(self) {
        loop {
            if self.cancellation_token.is_cancelled() && self.storage_request_receiver.is_empty() {
                info!("cancellation token is cancelled and storage request receiver is empty, storage dispatch is stopping");
                break;
            }

            let storage_request = match self.storage_request_receiver.recv().await {
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
                        &storage_request
                    );
                    continue;
                }
            };

            if let Err(error) = executor(storage_request).await {
                info!("storage executor returned error: {}", &error);
            }
        }
    }
}
