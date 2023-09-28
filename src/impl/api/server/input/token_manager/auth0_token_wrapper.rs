use std::{collections::HashMap, sync::Arc};

use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    DecodingKey, Validation,
};
use serde_json::Value;

use crate::{
    api::server::input::plugins::token_manager::{token::Token, token_wrapper::TokenWrapper},
    core::error::Error,
    core::error::ErrorKind,
    r#impl::api::server::input::token_manager::{
        json_web_token::JsonWebToken, open_id_connect_config::OpenIdConnectConfig,
    },
};

pub struct Auth0TokenWrapper {
    jwks: JwkSet,
    open_id_connect_config: OpenIdConnectConfig,
}

impl Auth0TokenWrapper {
    pub async fn try_new(open_id_connect_config: OpenIdConnectConfig) -> Result<Self, Error> {
        let jwks = try_get_jwks(open_id_connect_config.jwks_uri()).await?;

        Ok(Self {
            jwks,
            open_id_connect_config,
        })
    }
}

impl TokenWrapper for Auth0TokenWrapper {
    fn wrap(&self, token: &str) -> Result<Arc<dyn Token + Send + Sync>, Error> {
        let header = match decode_header(token) {
            Ok(header) => header,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::RequestError,
                    format!("failure to decode token's header: {}", error),
                ));
            }
        };

        let kid = match header.kid {
            Some(k) => k,
            None => {
                return Err(Error::new(
                    ErrorKind::RequestError,
                    "failed to find token header's kid",
                ));
            }
        };

        let jwk = match self.jwks.find(&kid) {
            Some(jwk) => jwk,
            None => {
                return Err(Error::new(
                    ErrorKind::RequestError,
                    format!("failed to find jwk for kid '{}'", kid),
                ));
            }
        };

        let rsa = match jwk.algorithm {
            AlgorithmParameters::RSA(ref rsa) => rsa,
            _ => {
                return Err(Error::new(
                    ErrorKind::RequestError,
                    format!("expected 'RSA' algorithm got '{:?}'", jwk.algorithm),
                ));
            }
        };

        let decoding_key = match DecodingKey::from_rsa_components(&rsa.n, &rsa.e) {
            Ok(decoding_key) => decoding_key,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::RequestError,
                    format!("failed to get decoding key: {}", error),
                ));
            }
        };

        let algorithm = match jwk.common.algorithm {
            Some(algorithm) => algorithm,
            None => {
                return Err(Error::new(
                    ErrorKind::RequestError,
                    "jwk is missing algorithm",
                ));
            }
        };

        let mut validation = Validation::new(algorithm);
        validation.validate_exp = true;
        validation.set_audience(self.open_id_connect_config.audience());
        validation.set_issuer(self.open_id_connect_config.issuers());

        let decoded_token =
            match decode::<HashMap<String, Value>>(token, &decoding_key, &validation) {
                Ok(decoded_token) => decoded_token,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::RequestError,
                        format!("invalid token detected: {}", error),
                    ));
                }
            };

        let token = JsonWebToken::try_new(decoded_token)?;
        Ok(Arc::new(token))
    }
}

async fn try_get_jwks(jwks_uri: &str) -> Result<JwkSet, Error> {
    let jwks = match reqwest::get(jwks_uri).await {
        Ok(response) => match response.json::<JwkSet>().await {
            Ok(jwks) => jwks,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("failed to deserialize response as JwkSet: {}", error),
                ));
            }
        },
        Err(error) => {
            return Err(Error::new(
                ErrorKind::ApiError,
                format!("failed to request jwks: {}", error),
            ));
        }
    };

    Ok(jwks)
}
