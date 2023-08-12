use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_channel::Sender;
use serde_json::Value;

use crate::api::input::request::Request;
use crate::error::Error;

pub type AsyncCallback<LogicRequestType> = Arc<
    dyn Fn(
            Request,
            Sender<LogicRequestType>,
        ) -> Pin<Box<dyn Future<Output = Result<Value, Error>> + Send + Sync>>
        + Send
        + Sync,
>;
