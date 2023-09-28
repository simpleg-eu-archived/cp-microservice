use std::mem::Discriminant;
use std::{collections::HashMap, sync::Arc};

use async_channel::{Receiver, Sender};
use multiple_connections_lapin_wrapper::{
    amqp_wrapper::AmqpWrapper, config::amqp_connect_config::AmqpConnectConfig,
};

use crate::api::server::input::action::Action;
use crate::r#impl::api::shared::amqp_api_entry::AmqpApiEntry;
use crate::{
    api::server::input::input_plugin::InputPlugin,
    r#impl::api::{
        server::input::amqp_input::AmqpInput, shared::amqp_queue_consumer::AmqpQueueConsumer,
    },
};

pub struct ApiInitializationPackage<LogicRequestType: 'static + Send + Sync + std::fmt::Debug> {
    pub amqp_connection_file: String,
    pub amqp_api_file: String,
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
    pub storage_request_sender: Sender<StorageRequestType>,
}

pub async fn try_initialize_microservice<
    LogicRequestType: 'static + Send + Sync + std::fmt::Debug,
    StorageRequestType: 'static + Send + Sync + std::fmt::Debug,
>(
    api_initialization_package: ApiInitializationPackage<LogicRequestType>,
    logic_initialization_package: LogicInitializationPackage<LogicRequestType, StorageRequestType>,
) -> Result<(), std::io::Error> {
    let amqp_connect_config: AmqpConnectConfig =
        get_amqp_connect_config(api_initialization_package.amqp_connection_file)?;

    let amqp_wrapper = match AmqpWrapper::try_new(amqp_connect_config) {
        Ok(amqp_wrapper) => amqp_wrapper,
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to create AMQP wrapper: {}", error),
            ))
        }
    };

    let amqp_api = get_amqp_api(api_initialization_package.amqp_api_file)?;

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

    let logic_dispatch: crate::logic::dispatch::Dispatch<LogicRequestType, StorageRequestType> =
        crate::logic::dispatch::Dispatch::new(
            logic_request_receiver,
            logic_initialization_package.executors,
            logic_initialization_package.storage_request_sender,
        );

    tokio::spawn(logic_dispatch.run());

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

fn get_amqp_api<'a>(amqp_api_file: String) -> Result<Vec<AmqpApiEntry>, std::io::Error> {
    let amqp_api_file_content = std::fs::read_to_string(amqp_api_file)?;

    let amqp_api = match serde_json::from_str::<Vec<AmqpApiEntry>>(&amqp_api_file_content) {
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

async fn generate_inputs_from_api(
    mut amqp_wrapper: AmqpWrapper,
    amqp_api: Vec<AmqpApiEntry>,
) -> Result<Vec<AmqpInput>, std::io::Error> {
    let mut inputs: Vec<AmqpInput> = Vec::new();

    for amqp_api_entry in amqp_api {
        let channel = match amqp_wrapper.try_get_channel().await {
            Ok(channel) => channel,
            Err(error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("failed to get AMQP channel: {}", &error),
                ))
            }
        };

        let amqp_input = match AmqpInput::try_new(channel, amqp_api_entry.amqp_queue_consumer).await
        {
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
