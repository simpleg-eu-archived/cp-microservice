[![CI/CD](https://github.com/simpleg-eu/cp-microservice/actions/workflows/ci-cd.yml/badge.svg)](https://github.com/simpleg-eu/cp-microservice/actions/workflows/ci-cd.yml)

# Introduction

cp-microservice is meant to be a utility library so you can easily create microservices with Rust. Currently all effort is focused towards AMQP based APIs, although the library can easily be fit to expose REST APIs through HTTP.

## Architecture

The architecture proposed by cp-microservice for Rust microservices is designed around the idea of 3 layers which run in parallel. These layers are the following:

1. API: Here incoming requests are routed and handled accordingly by sending requests to the `Logic` layer.
2. Logic: The business logic resides here. Here the incoming logic requests are handled and whenever there's a need for storage related actions, requests are sent to the storage layer.
3. Storage: Here storage requests are handled by doing direct calls to the database or whatever storage system is being used.

## Getting started

In order to get started with this library you can use as a reference the following project [cp-organization](https://github.com/simpleg-eu/cp-organization). It contains the expected usage of this library. But here are the steps for successfully implementing for your microservice project:

1. Add `cp-microservice` as a dependency within your project by adding the following line to your `Cargo.toml`: `cp-microservice = "0.1"`.
2. Next, we create a new file `api_actions.rs` within the `api` module (`src/api`).
   The content of the `api_actions.rs` file is a public function `get_api_actions` which will return all the actions which are available through the API.
   Here's an example from `cp-organization` of the `api_actions.rs` file:
   
   ```rust
    use std::{collections::HashMap, sync::Arc};
  
    use cp_microservice::api::server::input::action::Action;
    
    use crate::logic::logic_request::LogicRequest;
    
    pub fn get_api_actions() -> HashMap<String, Action<LogicRequest>> {
        let mut actions: HashMap<String, Action<LogicRequest>> = HashMap::new();
    
        actions.insert(
            "create_org".to_string(),
            Action::new(
                "create_org".to_string(),
                Arc::new(move |request, sender| {
                    Box::pin(crate::api::actions::create_org::create_org(request, sender))
                }),
                Vec::new(),
            ),
        );
    
        actions.insert(
            "create_invitation_code".to_string(),
            Action::new(
                "create_invitation_code".to_string(),
                Arc::new(move |request, sender| {
                    Box::pin(
                        crate::api::actions::create_invitation_code::create_invitation_code(
                            request, sender,
                        ),
                    )
                }),
                Vec::new(),
            ),
        );
    
        actions
    }
   ```
3. Next, we can define custom plugins for defining custom behaviours regarding the handling of incoming requests through the exposed API. The custom plugins must be listed in the `api_plugins.rs` file which must be contained within the `api` module (`src/api`). Here's an example from `cp-organization`:

   ```rust
    use std::sync::Arc;

    use cp_microservice::{api::server::input::input_plugin::InputPlugin, core::error::Error};
    
    pub async fn get_api_plugins() -> Result<Vec<Arc<dyn InputPlugin + Send + Sync>>, Error> {
        let api_plugins: Vec<Arc<dyn InputPlugin + Send + Sync>> = vec![];
    
        Ok(api_plugins)
    }
   ```
4. Now that we have defined the API's actions and plugins. We can proceed to initialize our microservice within the ´main.rs´ of our microservice. Here's the minimum code required for initializing a microservice with `cp-microservice`:
   ```rust
    let secrets_manager: Arc<dyn SecretsManager> = get_secrets_manager()?;
    let amqp_connection_config = get_amqp_connection_config(&secrets_manager)?;
    let amqp_api = get_amqp_api()?;

    let api_actions: HashMap<String, Action<LogicRequest>> = get_api_actions();

    let api_plugins: Vec<Arc<dyn InputPlugin + Send + Sync>> = match get_api_plugins().await {
        Ok(api_plugins) => api_plugins,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("failed to get API plugins: {}", &error),
            ))
        }
    };

    let api_initialization_package = ApiInitializationPackage::<LogicRequest> {
        amqp_connection_config,
        amqp_api,
        actions: api_actions,
        plugins: api_plugins,
    };

    let logic_executors = get_logic_executors();

    let (storage_request_sender, storage_request_receiver) =
        async_channel::bounded::<StorageRequest>(1024usize);

    let logic_initialization_package = LogicInitializationPackage::<LogicRequest, StorageRequest> {
        executors: logic_executors,
        storage_request_sender,
    };

    match try_initialize_microservice(api_initialization_package, logic_initialization_package)
        .await
    {
        Ok(_) => (),
        Err(error) => return Err(error),
    };
   ```
   
   The initialization functions called within the previous code can be stored for example within a `init.rs` file like in ´cp-organization´:
   
   ```rust
    use std::{
        io::{Error, ErrorKind},
        sync::Arc,
    };
    
    use cp_microservice::{
        core::secrets::secrets_manager::SecretsManager,
        r#impl::{
            api::shared::amqp_api_entry::AmqpApiEntry,
            core::bitwarden_secrets_manager::BitwardenSecretsManager,
        },
    };
    use mongodb::{options::ClientOptions, Client};
    use multiple_connections_lapin_wrapper::config::amqp_connect_config::AmqpConnectConfig;
    
    const SECRETS_MANAGER_ACCESS_TOKEN_ENV: &str = "CP_ORGANIZATION_SECRETS_MANAGER_ACCESS_TOKEN";
    const AMQP_CONNECTION_CONFIG_SECRET_ENV: &str = "CP_ORGANIZATION_AMQP_CONNECTION_SECRET";
    const MONGODB_CONNECTION_CONFIG_SECRET_ENV: &str = "CP_ORGANIZATION_MONGODB_CONNECTION_SECRET";
    
    pub fn get_secrets_manager() -> Result<Arc<dyn SecretsManager>, Error> {
        let access_token = match std::env::var(SECRETS_MANAGER_ACCESS_TOKEN_ENV) {
            Ok(access_token) => access_token,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "no access token provided",
                ));
            }
        };
    
        Ok(Arc::new(BitwardenSecretsManager::new(access_token)))
    }
    
    pub fn get_amqp_connection_config(
        secrets_manager: &Arc<dyn SecretsManager>,
    ) -> Result<AmqpConnectConfig, Error> {
        let secret_id = match std::env::var(AMQP_CONNECTION_CONFIG_SECRET_ENV) {
            Ok(secret_id) => secret_id,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "failed to read secret id '{}'",
                        AMQP_CONNECTION_CONFIG_SECRET_ENV
                    ),
                ));
            }
        };
    
        let amqp_connection_config_json = match secrets_manager.get(&secret_id) {
            Some(amqp_connection_config_json) => amqp_connection_config_json,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("no secret with id '{}'", &secret_id),
                ));
            }
        };
    
        let amqp_connection_config =
            match serde_json::from_str::<AmqpConnectConfig>(&amqp_connection_config_json) {
                Ok(amqp_connection_config) => amqp_connection_config,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("secret contains invalid amqp connection config: {}", &error),
                    ));
                }
            };
    
        Ok(amqp_connection_config)
    }
    
    pub fn get_amqp_api() -> Result<Vec<AmqpApiEntry>, Error> {
        let amqp_api_file = match std::env::args().nth(1) {
            Some(amqp_api_file) => amqp_api_file,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "no amqp api file provided",
                ));
            }
        };
    
        let amqp_api_file_content = match std::fs::read_to_string(&amqp_api_file) {
            Ok(content) => content,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "failed to find amqp api file '{}': {}",
                        &amqp_api_file, &error
                    ),
                ))
            }
        };
    
        let amqp_api = match serde_json::from_str::<Vec<AmqpApiEntry>>(&amqp_api_file_content) {
            Ok(amqp_api) => amqp_api,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("failed to deserialize AMQP API file: {}", &error),
                ))
            }
        };
    
        Ok(amqp_api)
    }
    
    pub fn get_mongodb_client(secrets_manager: &Arc<dyn SecretsManager>) -> Result<Client, Error> {
        let secret_id = match std::env::var(MONGODB_CONNECTION_CONFIG_SECRET_ENV) {
            Ok(secret_id) => secret_id,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "failed to read secret id '{}'",
                        MONGODB_CONNECTION_CONFIG_SECRET_ENV
                    ),
                ));
            }
        };
    
        let mongodb_connection_config_json = match secrets_manager.get(&secret_id) {
            Some(mongodb_connection_config_json) => mongodb_connection_config_json,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("no secret with id '{}'", &secret_id),
                ));
            }
        };
    
        let mongodb_client_options =
            match serde_json::from_str::<ClientOptions>(&mongodb_connection_config_json) {
                Ok(mongodb_client_options) => mongodb_client_options,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "failed to deserialize MongoDB connection config: {}",
                            &error
                        ),
                    ))
                }
            };
    
        let mongodb_client = match Client::with_options(mongodb_client_options) {
            Ok(mongodb_client) => mongodb_client,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("failed to create MongoDB client: {}", &error),
                ))
            }
        };
    
        Ok(mongodb_client)
    }

   ```
That would be it for configuring a basic microservice with `cp-microservice`.

## Objective

The current objective of `cp-microservice` is to make the creation of microservices, which usually are created with Spring Boot or similar technologies, easily manageable with Rust.
