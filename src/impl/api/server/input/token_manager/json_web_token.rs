use std::collections::HashMap;

use jsonwebtoken::TokenData;
use serde_json::Value;

use crate::{
    api::server::input::plugins::token_manager::token::Token,
    core::error::{Error, ErrorKind},
};

const AUTH0_PERMISSIONS_CLAIM: &str = "permissions";
const ORGANIZATION_PERMISSIONS_CLAIM: &str = "org_permissions";

const USER_ID_CLAIM: &str = "sub";

pub struct JsonWebToken {
    data: TokenData<HashMap<String, Value>>,
    permissions: Vec<String>,
    user_id: String,
}

impl JsonWebToken {
    pub fn try_new(data: TokenData<HashMap<String, Value>>) -> Result<Self, Error> {
        let mut permissions = get_permissions_from_claim(&data, AUTH0_PERMISSIONS_CLAIM)?;

        let mut organization_permissions =
            get_permissions_from_claim(&data, ORGANIZATION_PERMISSIONS_CLAIM)?;

        permissions.append(&mut organization_permissions);

        let user_id = match data.claims.get(USER_ID_CLAIM) {
            Some(id) => match id.as_str() {
                Some(id) => id.to_string(),
                None => {
                    return Err(Error::new(
                        ErrorKind::ApiError,
                        "failed to read 'sub' claim as a string",
                    ))
                }
            },
            None => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    "'JsonWebToken' is missing 'sub' claim",
                ))
            }
        };

        Ok(Self {
            data,
            permissions,
            user_id,
        })
    }
}

impl Token for JsonWebToken {
    fn can_execute(&self, action: &str) -> bool {
        self.permissions.contains(&action.to_string())
    }

    fn user_id(&self) -> &str {
        self.user_id.as_str()
    }
}

fn get_permissions_from_claim(
    data: &TokenData<HashMap<String, Value>>,
    claim: &str,
) -> Result<Vec<String>, Error> {
    let permissions = match data.claims.get(claim) {
        Some(permissions) => match serde_json::from_value::<Vec<String>>(permissions.clone()) {
            Ok(permissions) => permissions,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!(
                        "failed to deserialize permissions as a strings vector: {}",
                        error
                    ),
                ))
            }
        },
        None => {
            return Err(Error::new(
                ErrorKind::ApiError,
                format!("'{}' claim is missing", claim),
            ))
        }
    };

    Ok(permissions)
}
