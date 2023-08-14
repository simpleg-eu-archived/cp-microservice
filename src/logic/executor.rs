use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_channel::Sender;

use crate::error::Error;

pub type Executor<LogicRequestType, StorageRequestType> = Arc<
    dyn Fn(
            LogicRequestType,
            Sender<StorageRequestType>,
        ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>
        + Send
        + Sync,
>;
