use std::sync::Arc;

use async_trait::async_trait;

use crate::api::server::input::input_data::InputData;
use crate::api::server::input::plugins::token_manager::token::Token;
use crate::core::error::Error;

#[async_trait]
pub trait TokenManagerPlugin {
    async fn handle_input_data_with_token(
        &self,
        input_data: InputData,
        token: Arc<dyn Token + Send + Sync>,
    ) -> Result<InputData, (InputData, Error)>;
}
