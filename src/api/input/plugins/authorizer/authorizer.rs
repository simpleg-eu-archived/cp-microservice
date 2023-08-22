use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;
use tokio::time::timeout;

use crate::api::input::input_data::InputData;
use crate::api::input::input_plugin::InputPlugin;
use crate::api::input::plugins::authorizer::token_validator::TokenValidator;
use crate::api::input::replier::Replier;
use crate::api::input::request::Request;
use crate::api::input::request_header::RequestHeader;
use crate::error::{Error, ErrorKind};

pub struct Authorizer {
    token_validator: Arc<dyn TokenValidator + Send + Sync>,
}

impl Authorizer {
    pub fn new(token_validator: Arc<dyn TokenValidator + Send + Sync>) -> Authorizer {
        Authorizer { token_validator }
    }
}

#[async_trait]
impl InputPlugin for Authorizer {
    async fn handle_input_data(&self, input_data: InputData) -> Result<InputData, Error> {
        match self
            .token_validator
            .validate(input_data.request.header().token())
        {
            Ok(_) => Ok(input_data),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
fn create_dummy_input_data() -> InputData {
    let action: String = "abcd".to_string();
    let token: String = "192JFASNI349329".to_string();

    let request_header: RequestHeader = RequestHeader::new(action, token);
    let replier: Replier = Arc::new(move |value| Box::pin(async { Ok(()) }));

    let request = Request::new(request_header, Value::Null);

    InputData { request, replier }
}

#[derive(Default)]
pub struct AlwaysFailingTokenValidator {}

impl TokenValidator for AlwaysFailingTokenValidator {
    fn validate(&self, token: &str) -> Result<(), Error> {
        Err(Error::new(ErrorKind::RequestError, "token is invalid"))
    }
}

#[tokio::test]
pub async fn uses_passed_token_validator() {
    let token_validator: Arc<dyn TokenValidator + Send + Sync> =
        Arc::new(AlwaysFailingTokenValidator::default());
    let authorizer: Authorizer = Authorizer::new(token_validator);
    let example_input_data: InputData = create_dummy_input_data();

    let error = match timeout(
        Duration::from_millis(200u64),
        authorizer.handle_input_data(example_input_data),
    )
    .await
    .unwrap()
    {
        Ok(_) => panic!("expected error"),
        Err(error) => error,
    };

    assert_eq!(ErrorKind::RequestError, error.kind());
}
