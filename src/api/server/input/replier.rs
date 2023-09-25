use std::{future::Future, pin::Pin, sync::Arc};

use serde_json::Value;

use crate::core::error::Error;

pub type Replier = Arc<
    dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + Sync>> + Send + Sync,
>;
