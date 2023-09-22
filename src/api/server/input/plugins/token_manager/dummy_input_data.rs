#[cfg(test)]
use std::sync::Arc;

use serde_json::Value;

use crate::api::server::input::input_data::InputData;
use crate::api::{shared::{request_header::RequestHeader, request::Request}, server::input::replier::Replier};

pub fn create_dummy_input_data() -> InputData {
    let action: String = "abcd".to_string();
    let token: String = "192JFASNI349329".to_string();

    let request_header: RequestHeader = RequestHeader::new(action, token);
    let replier: Replier = Arc::new(move |value| Box::pin(async { Ok(()) }));

    let request = Request::new(request_header, Value::Null);

    InputData { request, replier }
}
