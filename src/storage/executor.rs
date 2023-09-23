use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::error::Error;

pub type Executor<StorageConnectionType, StorageRequestType> = Arc<
    dyn Fn(
            StorageConnectionType,
            StorageRequestType,
        ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>
        + Send
        + Sync,
>;
