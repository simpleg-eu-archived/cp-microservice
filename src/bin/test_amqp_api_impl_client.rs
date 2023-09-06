use std::sync::Arc;

use lapin::Channel;
use multiple_connections_lapin_wrapper::amqp_wrapper::AmqpWrapper;
use multiple_connections_lapin_wrapper::config::amqp_connect_config::AmqpConnectConfig;
use serde_json::{json, Value};

use cp_microservice::api::client::input_consumer::input_consumer::InputConsumer;
use cp_microservice::api::shared::request::Request;
use cp_microservice::api::shared::request_header::RequestHeader;
use cp_microservice::r#impl::api::client::input_consumer::amqp_input_consumer::AmqpInputConsumer;
use cp_microservice::r#impl::api::shared::amqp_queue_rpc_publisher::AmqpQueueRpcPublisher;

#[tokio::main]
pub async fn main() {
    let amqp_connection_json: &str = r#"{
                                            "uri": "amqp://guest:guest@127.0.0.1:5672",
                                            "options": {
                                                "locale": "en_US",
                                                "client_properties": {}
                                            },
                                            "owned_tls_config": {}
                                        }"#;

    let connection_config: AmqpConnectConfig =
        serde_json::from_str(amqp_connection_json).expect("expected connection config");
    let mut wrapper: AmqpWrapper = AmqpWrapper::try_new(connection_config)
        .expect("expected amqp wrapper from connection config");

    let channel: Arc<Channel> = wrapper
        .try_get_channel()
        .await
        .expect("expected amqp channel");

    let amqp_publisher_json: &str = r#"{
                                            "queue_name": "dummy",
                                            "publish": {
                                                "exchange": "",
                                                "options": {
                                                    "mandatory": false,
                                                    "immediate": false
                                                },
                                                "properties": {
                                                    "correlation_id": "1"
                                                }
                                            },
                                            "response": {
                                                "queue": {
                                                    "name": "",
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
                                                    "prefetch_count": 16,
                                                    "options": {
                                                        "global": false
                                                    }
                                                },
                                                "consume": {
                                                    "options": {
                                                        "no_local": false,
                                                        "no_ack": false,
                                                        "exclusive": false,
                                                        "nowait": false
                                                    },
                                                    "arguments": {}
                                                },
                                                "acknowledge": {
                                                    "multiple": false
                                                },
                                                "reject": {
                                                    "requeue": false
                                                }
                                            }
                                       }"#;

    let publisher: AmqpQueueRpcPublisher =
        serde_json::from_str::<AmqpQueueRpcPublisher>(amqp_publisher_json).unwrap();

    let amqp_input_consumer: AmqpInputConsumer =
        AmqpInputConsumer::new(channel, publisher, 50000u64);
    let request: Request = Request::new(
        RequestHeader::new("dummy:action".to_string(), "".to_string()),
        json!("expected"),
    );

    let response: Value = amqp_input_consumer.send_request(request).await.unwrap();
    let response_object = response.as_object().unwrap();

    let response_ok = response_object.get("Ok").unwrap();
    let response_string = response_ok.as_str().unwrap();

    assert_eq!("expected", response_string);
}
