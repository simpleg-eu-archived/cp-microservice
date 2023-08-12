use crate::{api::input::input_data::InputData, error::Error};

///
/// Entry point, for requests, into the server's logic.
///
#[async_trait::async_trait]
pub trait Input {
    async fn receive(&self) -> Result<InputData, Error>;
}
