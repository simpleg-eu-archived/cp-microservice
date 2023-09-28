use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_channel::Sender;
use serde_json::Value;

use crate::api::shared::request::Request;
use crate::core::error::Error;

pub type Executor<LogicRequestType> = Arc<
    dyn Fn(
            Request,
            Sender<LogicRequestType>,
        ) -> Pin<Box<dyn Future<Output = Result<Value, Error>> + Send + Sync>>
        + Send
        + Sync,
>;
