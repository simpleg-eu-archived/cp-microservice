use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct OpenIdConnectConfig {
    jwks_uri: String,
    issuers: Vec<String>,
    audience: Vec<String>,
}

impl OpenIdConnectConfig {
    pub fn new(jwks_uri: String, issuers: Vec<String>, audience: Vec<String>) -> Self {
        Self {
            jwks_uri,
            issuers,
            audience,
        }
    }

    pub fn jwks_uri(&self) -> &str {
        self.jwks_uri.as_str()
    }

    pub fn issuers(&self) -> &[String] {
        self.issuers.as_slice()
    }

    pub fn audience(&self) -> &[String] {
        self.audience.as_slice()
    }
}
