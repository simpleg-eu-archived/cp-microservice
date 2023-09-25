use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::api::server::input::input_data::InputData;
use crate::api::server::input::input_plugin::InputPlugin;
use crate::api::server::input::plugins::token_manager::authenticator::authenticator::Authenticator;
use crate::api::server::input::plugins::token_manager::authorizer::authorizer::Authorizer;
use crate::api::server::input::plugins::token_manager::token::Token;
use crate::api::server::input::plugins::token_manager::token_manager_plugin::TokenManagerPlugin;
use crate::api::server::input::plugins::token_manager::token_wrapper::TokenWrapper;
use crate::api::server::input::replier::Replier;
use crate::api::shared::request::Request;
use crate::api::shared::request_header::RequestHeader;
use crate::core::error::{Error, ErrorKind};

pub const TOKEN_MANAGER_PLUGIN_ID: &str = "token_manager";

pub struct TokenManager {
    token_wrapper: Arc<dyn TokenWrapper + Send + Sync>,
    authorizer: Arc<dyn TokenManagerPlugin + Send + Sync>,
    authenticator: Arc<dyn TokenManagerPlugin + Send + Sync>,
}

impl TokenManager {
    pub fn new(
        token_wrapper: Arc<dyn TokenWrapper + Send + Sync>,
        authorizer: Authorizer,
        authenticator: Authenticator,
    ) -> TokenManager {
        TokenManager {
            token_wrapper,
            authorizer: Arc::new(authorizer),
            authenticator: Arc::new(authenticator),
        }
    }
}

#[async_trait]
impl InputPlugin for TokenManager {
    fn id(&self) -> &str {
        TOKEN_MANAGER_PLUGIN_ID
    }

    async fn handle_input_data(
        &self,
        mut input_data: InputData,
    ) -> Result<InputData, (InputData, Error)> {
        let token: Arc<dyn Token + Send + Sync> =
            match self.token_wrapper.wrap(input_data.request.header().token()) {
                Ok(token) => token,
                Err(error) => return Err((input_data, error)),
            };

        input_data = match self
            .authorizer
            .handle_input_data_with_token(input_data, token.clone())
            .await
        {
            Ok(input_data) => input_data,
            Err((input_data, error)) => return Err((input_data, error)),
        };

        input_data = match self
            .authenticator
            .handle_input_data_with_token(input_data, token)
            .await
        {
            Ok(input_data) => input_data,
            Err((input_data, error)) => return Err((input_data, error)),
        };

        Ok(input_data)
    }
}

#[cfg(test)]
use crate::api::server::input::plugins::token_manager::dummy_input_data::create_dummy_input_data;

#[tokio::test]
pub async fn error_when_token_wrapper_fails() {
    let token_wrapper: Arc<dyn TokenWrapper + Send + Sync> =
        Arc::new(AlwaysFailingTokenWrapper::default());

    let token_manager: TokenManager = TokenManager::new(
        token_wrapper,
        Authorizer::default(),
        Authenticator::default(),
    );
    let dummy_input_data: InputData = create_dummy_input_data();

    match token_manager.handle_input_data(dummy_input_data).await {
        Ok(_) => panic!("expected 'Err' got 'Ok'"),
        Err((_input_data, error)) => assert_eq!(ErrorKind::ApiError, error.kind),
    }
}

#[tokio::test]
pub async fn error_when_token_authorization_fails() {
    let token_wrapper: Arc<dyn TokenWrapper + Send + Sync> =
        Arc::new(NoPermissionTokenWrapper::default());

    let token_manager: TokenManager = TokenManager::new(
        token_wrapper,
        Authorizer::default(),
        Authenticator::default(),
    );
    let dummy_input_data: InputData = create_dummy_input_data();

    match token_manager.handle_input_data(dummy_input_data).await {
        Ok(_) => panic!("expected 'Err' got 'Ok'"),
        Err((_input_data, error)) => assert_eq!(ErrorKind::RequestError, error.kind),
    }
}

#[derive(Default)]
pub struct AlwaysFailingTokenWrapper {}

impl TokenWrapper for AlwaysFailingTokenWrapper {
    fn wrap(&self, _token: &str) -> Result<Arc<dyn Token + Send + Sync>, Error> {
        Err(Error::new(
            ErrorKind::ApiError,
            "failed to initialize token",
        ))
    }
}

#[derive(Default)]
pub struct NoPermissionTokenWrapper {}

impl TokenWrapper for NoPermissionTokenWrapper {
    fn wrap(&self, _token: &str) -> Result<Arc<dyn Token + Send + Sync>, Error> {
        Ok(Arc::new(NoPermissionToken::default()))
    }
}

#[derive(Default)]
pub struct NoPermissionToken {}

impl Token for NoPermissionToken {
    fn can_execute(&self, _action: &str) -> bool {
        false
    }

    fn user_id(&self) -> &str {
        todo!()
    }
}

#[derive(Default)]
pub struct AllPermissionsTokenWrapper {}

impl TokenWrapper for AllPermissionsTokenWrapper {
    fn wrap(&self, _token: &str) -> Result<Arc<dyn Token + Send + Sync>, Error> {
        Ok(Arc::new(AllPermissionsToken::default()))
    }
}

#[derive(Default)]
pub struct AllPermissionsToken {}

impl Token for AllPermissionsToken {
    fn can_execute(&self, _action: &str) -> bool {
        true
    }

    fn user_id(&self) -> &str {
        ""
    }
}
