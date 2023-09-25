use std::collections::HashMap;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use async_channel::Sender;
use lapin::Channel;
use multiple_connections_lapin_wrapper::amqp_wrapper::AmqpWrapper;
use multiple_connections_lapin_wrapper::config::amqp_connect_config::AmqpConnectConfig;
use serde_json::Value;

use cp_microservice::api::server::action::Action;
use cp_microservice::api::server::dispatch::Dispatch;
use cp_microservice::api::server::input::input_plugin::InputPlugin;
use cp_microservice::api::shared::request::Request;
use cp_microservice::core::error::Error;
use cp_microservice::r#impl::api::server::input::amqp_input::AmqpInput;
use cp_microservice::r#impl::api::shared::amqp_queue_consumer::AmqpQueueConsumer;

const ALIVE_TIME_IN_MILLISECONDS: u64 = 600000u64;

#[tokio::main]
pub async fn main() {
    let amqp_connection_uri = std::env::args()
        .nth(1usize)
        .expect("expected amqp connection uri");

    let amqp_connection_json: String = format!("{{ \"uri\": \"{}\", \"options\": {{ \"locale\": \"en_US\", \"client_properties\": {{}} }},\"owned_tls_config\": {{}} }}", amqp_connection_uri);

    let connection_config: AmqpConnectConfig =
        serde_json::from_str(amqp_connection_json.as_str()).expect("expected connection config");
    let mut wrapper: AmqpWrapper = AmqpWrapper::try_new(connection_config)
        .expect("expected amqp wrapper from connection config");

    let channel: Arc<Channel> = wrapper
        .try_get_channel()
        .await
        .expect("expected amqp channel");

    let amqp_queue_consumer_json: &str = r#"{
        "queue": {
          "name": "dummy",
          "declare": {
            "options": {
              "passive": false,
              "durable": false,
              "exclusive": false,
              "auto_delete": true,
              "nowait": false
            },
            "arguments": {}
          }
        },
        "qos": {
          "prefetch_count": 10,
          "options": {
            "global": true
          }
        },
        "consume": {
          "options": {
            "no_local": true,
            "no_ack": false,
            "exclusive": true,
            "nowait": false
          },
          "arguments": {
          }
        },
        "acknowledge": {
          "multiple": false
        },
        "reject": {
          "requeue": false
        }
      }"#;

    let amqp_queue_consumer: AmqpQueueConsumer =
        serde_json::from_str(amqp_queue_consumer_json).expect("expected amqp queue consumer");

    let amqp_input: AmqpInput = AmqpInput::try_new(channel, amqp_queue_consumer, Vec::new())
        .await
        .unwrap();
    let inputs = vec![amqp_input];
    let dummy_action: Action<DummyLogicRequest> =
        Arc::new(move |request, sender| Box::pin(dummy_action(request, sender)));
    let actions: HashMap<String, Action<DummyLogicRequest>> =
        HashMap::from([("dummy:action".to_string(), dummy_action)]);

    let (sender, _receiver) = async_channel::unbounded::<DummyLogicRequest>();
    let plugins: Vec<Arc<dyn InputPlugin + Send + Sync>> = Vec::new();

    let dispatch: Dispatch<AmqpInput, DummyLogicRequest> =
        Dispatch::new(inputs, actions, sender, plugins);

    tokio::spawn(dispatch.run());

    sleep(Duration::from_millis(ALIVE_TIME_IN_MILLISECONDS));
}

pub enum DummyLogicRequest {}

pub async fn dummy_action(
    request: Request,
    _sender: Sender<DummyLogicRequest>,
) -> Result<Value, Error> {
    Ok(request.payload().clone())
}
