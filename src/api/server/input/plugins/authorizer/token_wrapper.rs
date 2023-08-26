use crate::api::server::input::plugins::authorizer::token::Token;
use std::sync::Arc;

use crate::error::Error;

pub trait TokenWrapper {
    fn wrap(&self, token: &str) -> Result<Arc<dyn Token + Send + Sync>, Error>;
}
