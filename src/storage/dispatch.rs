use std::{
    collections::HashMap,
    fmt::Debug,
    mem::{self, Discriminant},
};

use async_channel::Receiver;
use log::info;

use crate::storage::executor::Executor;

pub struct Dispatch<StorageRequestType: Debug> {
    storage_request_receiver: Receiver<StorageRequestType>,
    executors: HashMap<Discriminant<StorageRequestType>, Executor<StorageRequestType>>,
}

impl<StorageRequestType: Debug> Dispatch<StorageRequestType> {
    pub fn new(
        storage_request_receiver: Receiver<StorageRequestType>,
        executors: HashMap<Discriminant<StorageRequestType>, Executor<StorageRequestType>>,
    ) -> Self {
        Self {
            storage_request_receiver,
            executors,
        }
    }

    pub async fn run(self) {
        loop {
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
