use crate::api::server::input::input_data::InputData;
use crate::core::error::Error;

///
/// Entry point, for requests, into the server's logic.
///
#[async_trait::async_trait]
pub trait Input {
    async fn receive(&mut self) -> Result<InputData, Error>;
}
