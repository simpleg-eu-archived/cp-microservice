use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::api::input::input_data::InputData;
use crate::error::Error;

pub type InputPlugin = Arc<
    dyn Fn(InputData) -> Pin<Box<dyn Future<Output = Result<InputData, Error>> + Send + Sync>>
        + Send
        + Sync,
>;
