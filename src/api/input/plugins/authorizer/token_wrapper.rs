use std::sync::Arc;

use crate::api::input::plugins::authorizer::token::Token;
use crate::error::Error;

pub trait TokenWrapper {
    fn wrap(&self, token: &str) -> Result<Arc<dyn Token + Send + Sync>, Error>;
}
