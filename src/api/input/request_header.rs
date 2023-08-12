use serde::Deserialize;

#[derive(Deserialize)]
pub struct RequestHeader {
    action: String,
}

impl RequestHeader {
    pub fn new(action: String) -> RequestHeader {
        RequestHeader { action }
    }

    pub fn action(&self) -> &str {
        self.action.as_str()
    }
}
