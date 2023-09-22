use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RequestHeader {
    action: String,
    token: String,
    extra: HashMap<String, String>,
}

impl RequestHeader {
    pub fn new(action: String, token: String) -> RequestHeader {
        RequestHeader {
            action,
            token,
            extra: HashMap::new(),
        }
    }

    pub fn action(&self) -> &str {
        self.action.as_str()
    }

    pub fn token(&self) -> &str {
        self.token.as_str()
    }

    pub fn add_extra(&mut self, key: String, value: String) -> Option<String> {
        self.extra.insert(key, value)
    }

    pub fn has_extra(&self, key: &String) -> bool {
        self.extra.contains_key(key)
    }

    pub fn get_extra(&self, key: &String) -> Option<&String> {
        self.extra.get(key)
    }
}
