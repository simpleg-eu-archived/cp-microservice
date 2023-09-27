use std::mem::Discriminant;
use std::{collections::HashMap, sync::Arc};

use mongodb::options::ClientOptions;
use mongodb::Client;
use multiple_connections_lapin_wrapper::{
    amqp_wrapper::AmqpWrapper, config::amqp_connect_config::AmqpConnectConfig,
};

use crate::{
    api::server::{action::Action, input::input_plugin::InputPlugin},
    r#impl::api::{
        server::input::amqp_input::AmqpInput, shared::amqp_queue_consumer::AmqpQueueConsumer,
    },
};

pub struct ApiInitializationPackage<LogicRequestType: 'static + Send + Sync + std::fmt::Debug> {
    pub actions: HashMap<String, Action<LogicRequestType>>,
    pub plugins: Vec<Arc<dyn InputPlugin + Send + Sync>>,
}

pub struct LogicInitializationPackage<
    LogicRequestType: 'static + Send + Sync + std::fmt::Debug,
    StorageRequestType: 'static + Send + Sync,
> {
    pub executors: HashMap<
        Discriminant<LogicRequestType>,
        crate::logic::executor::Executor<LogicRequestType, StorageRequestType>,
    >,
}

pub struct StorageInitializationPackage<
    StorageRequestType: 'static + Send + Sync + std::fmt::Debug,
    StorageConnectionType: 'static + Send + Sync + Clone,
> {
    pub executors: HashMap<
        Discriminant<StorageRequestType>,
        crate::storage::executor::Executor<StorageConnectionType, StorageRequestType>,
    >,
}

pub async fn try_initialize_microservice<
    LogicRequestType: 'static + Send + Sync + std::fmt::Debug,
    StorageRequestType: 'static + Send + Sync + std::fmt::Debug,
>(
    api_initialization_package: ApiInitializationPackage<LogicRequestType>,
    logic_initialization_package: LogicInitializationPackage<LogicRequestType, StorageRequestType>,
    storage_initialization_package: StorageInitializationPackage<StorageRequestType, Client>,
) -> Result<(), std::io::Error> {
    let mut args = std::env::args();

    let amqp_connection_file = match args.nth(1) {
        Some(amqp_connection_file) => amqp_connection_file,
        None => {
            log::error!("expected AMQP connection file as first argument");
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "expected AMQP connection file as first argument",
            ));
        }
    };

    let mongodb_connection_file = match args.nth(2) {
        Some(mongodb_connection_file) => mongodb_connection_file,
        None => {
            log::error!("expected MongoDB connection file as second argument");

            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "expected MongoDB connection file as first argument",
            ));
        }
    };

    let amqp_api_file = match args.nth(3) {
        Some(amqp_api_file) => amqp_api_file,
        None => {
            log::error!("expected AMQP API file as third argument");

            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "expected AMQP API file as third argument",
            ));
        }
    };

    let amqp_connect_config: AmqpConnectConfig = get_amqp_connect_config(amqp_connection_file)?;
    let mongodb_client_options: ClientOptions =
        get_mongodb_client_options(mongodb_connection_file)?;

    let amqp_wrapper = match AmqpWrapper::try_new(amqp_connect_config) {
        Ok(amqp_wrapper) => amqp_wrapper,
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to create AMQP wrapper: {}", error),
            ))
        }
    };

    let amqp_api = get_amqp_api(amqp_api_file)?;

    let amqp_inputs = generate_inputs_from_api(amqp_wrapper, amqp_api).await?;

    let (logic_request_sender, logic_request_receiver) =
        async_channel::bounded::<LogicRequestType>(1024usize);

    let api_dispatch: crate::api::server::dispatch::Dispatch<AmqpInput, LogicRequestType> =
        crate::api::server::dispatch::Dispatch::new(
            amqp_inputs,
            api_initialization_package.actions,
            logic_request_sender,
            api_initialization_package.plugins,
        );

    tokio::spawn(api_dispatch.run());

    let (storage_request_sender, storage_request_receiver) =
        async_channel::bounded::<StorageRequestType>(1024usize);

    let logic_dispatch: crate::logic::dispatch::Dispatch<LogicRequestType, StorageRequestType> =
        crate::logic::dispatch::Dispatch::new(
            logic_request_receiver,
            logic_initialization_package.executors,
            storage_request_sender,
        );

    tokio::spawn(logic_dispatch.run());

    let mongodb_client: Client = match Client::with_options(mongodb_client_options) {
        Ok(mongodb_client) => mongodb_client,
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to create MongoDB client: {}", error),
            ))
        }
    };

    let storage_dispatch: crate::storage::dispatch::Dispatch<StorageRequestType, Client> =
        crate::storage::dispatch::Dispatch::new(
            storage_request_receiver,
            storage_initialization_package.executors,
            mongodb_client,
        );

    tokio::spawn(storage_dispatch.run());

    Ok(())
}

fn get_amqp_connect_config(
    amqp_connection_file: String,
) -> Result<AmqpConnectConfig, std::io::Error> {
    let amqp_connection_file_content = std::fs::read_to_string(amqp_connection_file)?;

    let amqp_connect_config =
        match serde_json::from_str::<AmqpConnectConfig>(&amqp_connection_file_content) {
            Ok(amqp_connect_config) => amqp_connect_config,
            Err(error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("failed to deserialize AMQP connection file: {}", &error),
                ))
            }
        };

    Ok(amqp_connect_config)
}

fn get_mongodb_client_options(
    mongodb_connection_file: String,
) -> Result<ClientOptions, std::io::Error> {
    let mongodb_connection_file_content = std::fs::read_to_string(mongodb_connection_file)?;

    let mongodb_client_options =
        match serde_json::from_str::<ClientOptions>(&mongodb_connection_file_content) {
            Ok(mongodb_client_options) => mongodb_client_options,
            Err(error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("failed to deserialize MongoDB connection file: {}", &error),
                ))
            }
        };

    Ok(mongodb_client_options)
}

fn get_amqp_api(amqp_api_file: String) -> Result<Vec<AmqpQueueConsumer>, std::io::Error> {
    let amqp_api_file_content = std::fs::read_to_string(amqp_api_file)?;

    let amqp_api = match serde_json::from_str::<Vec<AmqpQueueConsumer>>(&amqp_api_file_content) {
        Ok(amqp_api) => amqp_api,
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("failed to deserialize AMQP API file: {}", &error),
            ))
        }
    };

    Ok(amqp_api)
}

async fn generate_inputs_from_api<'a>(
    mut amqp_wrapper: AmqpWrapper,
    amqp_api: Vec<AmqpQueueConsumer>,
) -> Result<Vec<AmqpInput<'a>>, std::io::Error> {
    let mut inputs: Vec<AmqpInput<'a>> = Vec::new();

    for amqp_queue_consumer in amqp_api {
        let channel = match amqp_wrapper.try_get_channel().await {
            Ok(channel) => channel,
            Err(error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("failed to get AMQP channel: {}", &error),
                ))
            }
        };

        let amqp_input = match AmqpInput::try_new(channel, amqp_queue_consumer, Vec::new()).await {
            Ok(amqp_input) => amqp_input,
            Err(error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("failed to create AMQP input: {}", &error),
                ))
            }
        };

        inputs.push(amqp_input);
    }

    Ok(inputs)
}
