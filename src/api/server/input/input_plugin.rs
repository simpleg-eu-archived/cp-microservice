use crate::api::server::input::input_data::InputData;
use async_trait::async_trait;

use crate::core::error::Error;

#[async_trait]
pub trait InputPlugin {
    fn id(&self) -> &str;
    async fn handle_input_data(
        &self,
        input_data: InputData,
    ) -> Result<InputData, (InputData, Error)>;
}
