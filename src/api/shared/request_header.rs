use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RequestHeader {
    action: String,
    token: String,
}

impl RequestHeader {
    pub fn new(action: String, token: String) -> RequestHeader {
        RequestHeader { action, token }
    }

    pub fn action(&self) -> &str {
        self.action.as_str()
    }

    pub fn token(&self) -> &str {
        self.token.as_str()
    }
}