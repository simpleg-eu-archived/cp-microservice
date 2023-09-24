use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::time::timeout;

use crate::api::server::input::input_data::InputData;
use crate::api::server::input::plugins::token_manager::token::Token;
use crate::api::server::input::plugins::token_manager::token_manager_plugin::TokenManagerPlugin;
use crate::error::{Error, ErrorKind};

#[derive(Default)]
pub struct Authorizer {}

#[async_trait]
impl TokenManagerPlugin for Authorizer {
    async fn handle_input_data_with_token(
        &self,
        input_data: InputData,
        token: Arc<dyn Token + Send + Sync>,
    ) -> Result<InputData, (InputData, Error)> {
        if !token.can_execute(input_data.request.header().action()) {
            return Err((
                input_data,
                Error::new(
                    ErrorKind::RequestError,
                    "token has no permission to execute action",
                ),
            ));
        }

        Ok(input_data)
    }
}

#[cfg(test)]
use crate::api::server::input::plugins::token_manager::dummy_input_data::create_dummy_input_data;

const TIMEOUT_AFTER_MILLISECONDS: u64 = 200u64;

#[tokio::test]
pub async fn fails_when_lacking_permission_for_action() {
    let authorizer: Authorizer = Authorizer::default();
    let example_input_data: InputData = create_dummy_input_data();
    let token: Arc<dyn Token + Send + Sync> = Arc::new(NoPermissionsToken::default());

    let error = match timeout(
        Duration::from_millis(TIMEOUT_AFTER_MILLISECONDS),
        authorizer.handle_input_data_with_token(example_input_data, token),
    )
    .await
    .unwrap()
    {
        Ok(_) => panic!("expected error"),
        Err((input_data, error)) => error,
    };

    assert_eq!(ErrorKind::RequestError, error.kind());
}

#[cfg(test)]
#[tokio::test]
pub async fn succeeds_when_can_execute_action() {
    let authorizer: Authorizer = Authorizer::default();
    let example_input_data: InputData = create_dummy_input_data();
    let token: Arc<dyn Token + Send + Sync> = Arc::new(AllPermissionsToken::default());

    match timeout(
        Duration::from_millis(TIMEOUT_AFTER_MILLISECONDS),
        authorizer.handle_input_data_with_token(example_input_data, token),
    )
    .await
    .unwrap()
    {
        Ok(_) => (),
        Err(_error) => panic!("expected 'Ok' for 'handle_input_data_with_token'."),
    };
}

#[derive(Default)]
pub struct NoPermissionsToken {}

impl Token for NoPermissionsToken {
    fn can_execute(&self, _action: &str) -> bool {
        false
    }

    fn user_id(&self) -> &str {
        todo!()
    }
}

#[derive(Default)]
pub struct AllPermissionsToken {}

impl Token for AllPermissionsToken {
    fn can_execute(&self, _action: &str) -> bool {
        true
    }

    fn user_id(&self) -> &str {
        todo!()
    }
}
