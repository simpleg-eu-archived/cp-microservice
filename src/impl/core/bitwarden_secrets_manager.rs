use std::process::Command;

use serde::Deserialize;

use crate::core::secrets::secrets_manager::SecretsManager;

#[derive(Deserialize)]
struct BitwardenSecret {
    pub object: String,
    pub id: String,
    pub organizationId: String,
    pub projectId: String,
    pub key: String,
    pub value: String,
    pub note: String,
    pub creationDate: String,
    pub revisionDate: String,
}

pub struct BitwardenSecretsManager {
    access_token: String,
}

impl BitwardenSecretsManager {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }
}

impl SecretsManager for BitwardenSecretsManager {
    fn get(&self, id: &str) -> Option<String> {
        let secret_data = match Command::new("bws")
            .arg("secret")
            .arg("get")
            .arg(id)
            .arg("--access-token")
            .arg(&self.access_token)
            .output()
        {
            Ok(secret_data) => secret_data.stdout,
            Err(error) => {
                log::warn!("failed to retrieve secret '{}': {}", id, &error);
                return None;
            }
        };

        let secret = match serde_json::from_slice::<BitwardenSecret>(&secret_data) {
            Ok(secret) => secret,
            Err(error) => {
                log::warn!("failed to deserialize secret: {}", &error);
                return None;
            }
        };

        Some(secret.value)
    }
}
