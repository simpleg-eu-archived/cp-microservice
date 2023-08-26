use crate::api::shared::request_header::RequestHeader;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Request {
    header: RequestHeader,
    payload: Value,
}

impl Request {
    pub fn new(header: RequestHeader, payload: Value) -> Request {
        Request { header, payload }
    }

    pub fn header(&self) -> &RequestHeader {
        &self.header
    }

    pub fn payload(&self) -> &Value {
        &self.payload
    }
}
