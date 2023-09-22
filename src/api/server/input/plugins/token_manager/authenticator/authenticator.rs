use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::time::timeout;

use crate::api::server::input::input_data::InputData;
use crate::api::server::input::plugins::token_manager::token::Token;
use crate::api::server::input::plugins::token_manager::token_manager_plugin::TokenManagerPlugin;
use crate::error::{Error, ErrorKind};

const USER_ID_KEY: &str = "user_id";

#[derive(Default)]
pub struct Authenticator {}

#[async_trait]
impl TokenManagerPlugin for Authenticator {
    async fn handle_input_data_with_token(
        &self,
        mut input_data: InputData,
        token: Arc<dyn Token + Send + Sync>,
    ) -> Result<InputData, Error> {
        let user_id = match token.user_id() {
            Some(user_id) => user_id,
            None => {
                return Err(Error::new(
                    ErrorKind::RequestError,
                    "token contains no 'user_id'",
                ))
            }
        };

        input_data
            .request
            .mut_header()
            .add_extra(USER_ID_KEY.to_string(), user_id);

        Ok(input_data)
    }
}

#[cfg(test)]
use crate::api::server::input::plugins::token_manager::dummy_input_data::create_dummy_input_data;

const TIMEOUT_AFTER_MILLISECONDS: u64 = 200u64;

#[tokio::test]
pub async fn error_if_user_id_is_missing() {
    let authenticator: Arc<dyn TokenManagerPlugin + Send + Sync> =
        Arc::new(Authenticator::default());
    let input_data = create_dummy_input_data();

    match timeout(
        Duration::from_millis(TIMEOUT_AFTER_MILLISECONDS),
        authenticator.handle_input_data_with_token(input_data, Arc::new(NoUserIdToken::default())),
    )
    .await
    .unwrap()
    {
        Ok(_) => panic!("expected 'Err' got 'Ok'"),
        Err(error) => assert_eq!(ErrorKind::RequestError, error.kind),
    }
}

#[tokio::test]
pub async fn embed_user_id_into_header() {
    let authenticator: Arc<dyn TokenManagerPlugin + Send + Sync> =
        Arc::new(Authenticator::default());
    let input_data: InputData = create_dummy_input_data();
    let token = Arc::new(TokenWithUserId::default());

    let result = match timeout(
        Duration::from_millis(TIMEOUT_AFTER_MILLISECONDS),
        authenticator.handle_input_data_with_token(input_data, token),
    )
    .await
    .unwrap()
    {
        Ok(input_data) => input_data,
        Err(error) => panic!("expected 'Ok' got an 'Err': {}", error),
    };

    assert!(result.request.header().has_extra(&USER_ID_KEY.to_string()));
}

#[derive(Default)]
pub struct NoUserIdToken {}

impl Token for NoUserIdToken {
    fn can_execute(&self, action: &str) -> bool {
        todo!()
    }

    fn user_id(&self) -> Option<String> {
        None
    }
}

#[derive(Default)]
pub struct TokenWithUserId {}

impl Token for TokenWithUserId {
    fn can_execute(&self, action: &str) -> bool {
        todo!()
    }

    fn user_id(&self) -> Option<String> {
        Some("123".to_string())
    }
}
