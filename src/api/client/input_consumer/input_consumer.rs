use async_trait::async_trait;
use serde_json::Value;

use crate::api::shared::request::Request;
use crate::error::Error;

#[async_trait]
pub trait InputConsumer {
    async fn send_request(&self, request: Request) -> Result<Value, Error>;
}
