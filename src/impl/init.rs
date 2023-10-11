use std::mem::Discriminant;
use std::{collections::HashMap, sync::Arc};

use async_channel::Sender;
use futures_util::future::join_all;
use multiple_connections_lapin_wrapper::{
    amqp_wrapper::AmqpWrapper, config::amqp_connect_config::AmqpConnectConfig,
};
use tokio_util::sync::CancellationToken;

use crate::api::server::input::action::Action;
use crate::r#impl::api::shared::amqp_api_entry::AmqpApiEntry;
use crate::r#impl::process_signals::listen_to_process_signals;
use crate::{
    api::server::input::input_plugin::InputPlugin,
    r#impl::api::server::input::amqp_input::AmqpInput,
};

pub struct ApiInitializationPackage<LogicRequestType: 'static + Send + Sync + std::fmt::Debug> {
    pub amqp_connection_config: AmqpConnectConfig,
    pub amqp_api: Vec<AmqpApiEntry>,
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
    let cancellation_token = CancellationToken::new();

    listen_to_process_signals(cancellation_token.clone());

    let amqp_wrapper = match AmqpWrapper::try_new(api_initialization_package.amqp_connection_config)
    {
        Ok(amqp_wrapper) => amqp_wrapper,
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to create AMQP wrapper: {}", error),
            ))
        }
    };

    let amqp_inputs =
        generate_inputs_from_api(amqp_wrapper, api_initialization_package.amqp_api).await?;

    let (logic_request_sender, logic_request_receiver) =
        async_channel::bounded::<LogicRequestType>(1024usize);

    let api_dispatch: crate::api::server::dispatch::Dispatch<AmqpInput, LogicRequestType> =
        crate::api::server::dispatch::Dispatch::new(
            amqp_inputs,
            api_initialization_package.actions,
            logic_request_sender,
            api_initialization_package.plugins,
        );

    let api_cancellation_token = cancellation_token.clone();
    tokio::spawn(async move {
        // when handles have finished, the program will exit since an exit signal is sent to the process
        let handles = api_dispatch.run(api_cancellation_token).await;

        join_all(handles).await;

        std::process::exit(0);
    });

    let logic_dispatch: crate::logic::dispatch::Dispatch<LogicRequestType, StorageRequestType> =
        crate::logic::dispatch::Dispatch::new(
            logic_request_receiver,
            logic_initialization_package.executors,
            logic_initialization_package.storage_request_sender,
            cancellation_token,
        );

    tokio::spawn(logic_dispatch.run());

    Ok(())
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
