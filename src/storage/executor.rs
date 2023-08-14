use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::error::Error;

pub type Executor<StorageRequestType> = Arc<
    dyn Fn(StorageRequestType) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>
        + Send
        + Sync,
>;
