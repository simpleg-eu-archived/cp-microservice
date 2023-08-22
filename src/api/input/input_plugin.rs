use async_trait::async_trait;

use crate::api::input::input_data::InputData;
use crate::error::Error;

#[async_trait]
pub trait InputPlugin {
    async fn handle_input_data(&self, input_data: InputData) -> Result<InputData, Error>;
}