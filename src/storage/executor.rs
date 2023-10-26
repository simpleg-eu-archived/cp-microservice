use std::{pin::Pin, sync::Arc};

use futures_util::Future;

use crate::core::error::Error;

pub type Executor<StorageRequestType> = Arc<
    dyn Fn(StorageRequestType) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>>
        + Send
        + Sync,
>;
