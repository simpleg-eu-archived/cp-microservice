use std::sync::Arc;

use crate::api::server::input::plugins::token_manager::token::Token;
use crate::core::error::Error;

pub trait TokenWrapper {
    fn wrap(&self, token: &str) -> Result<Arc<dyn Token + Send + Sync>, Error>;
}
