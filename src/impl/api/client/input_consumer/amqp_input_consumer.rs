use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::TryStreamExt;
use lapin::types::ShortString;
use lapin::Channel;
use serde_json::Value;
use tokio::time::timeout;

use crate::api::client::input_consumer::input_consumer::InputConsumer;
use crate::api::shared::request::Request;
use crate::error::{Error, ErrorKind};
use crate::r#impl::api::shared::amqp_queue_rpc_publisher::AmqpQueueRpcPublisher;

pub struct AmqpInputConsumer {
    channel: Arc<Channel>,
    publisher: AmqpQueueRpcPublisher,
    timeout_after: Duration,
}

impl AmqpInputConsumer {
    pub fn new(
        channel: Arc<Channel>,
        publisher: AmqpQueueRpcPublisher,
        timeout_after_milliseconds: u64,
    ) -> AmqpInputConsumer {
        AmqpInputConsumer {
            channel,
            publisher,
            timeout_after: Duration::from_millis(timeout_after_milliseconds),
        }
    }
}

#[async_trait]
impl InputConsumer for AmqpInputConsumer {
    async fn send_request(&self, request: Request) -> Result<Value, Error> {
        let request_payload = match serde_json::to_vec::<Request>(&request) {
            Ok(request_payload) => request_payload,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::RequestError,
                    format!("failed to serialize request: {}", error),
                ));
            }
        };

        let queue = match timeout(
            self.timeout_after,
            self.channel.queue_declare(
                self.publisher.response().queue().name(),
                self.publisher.response().queue().declare().options,
                self.publisher
                    .response()
                    .queue()
                    .declare()
                    .arguments
                    .clone(),
            ),
        )
        .await
        {
            Ok(result) => match result {
                Ok(queue) => queue,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::ApiError,
                        format!("failed to create response queue: {}", error),
                    ));
                }
            },
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("timed out creating response queue: {}", error),
                ));
            }
        };

        let reply_to: ShortString = queue.name().clone();
        let correlation_id: ShortString = ShortString::from(uuid::Uuid::new_v4().to_string());
        let properties = self
            .publisher
            .publish()
            .properties()
            .clone()
            .with_reply_to(reply_to)
            .with_correlation_id(correlation_id);

        match timeout(
            self.timeout_after,
            self.channel.basic_publish(
                self.publisher.publish().exchange(),
                self.publisher.queue_name(),
                *self.publisher.publish().options(),
                request_payload.as_slice(),
                properties,
            ),
        )
        .await
        {
            Ok(result) => {
                if let Err(error) = result {
                    return Err(Error::new(
                        ErrorKind::ApiError,
                        format!("failed to publish request: {}", error),
                    ));
                }
            }
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("timed out publishing request: {}", error),
                ));
            }
        }

        let mut consumer = match timeout(
            self.timeout_after,
            self.channel.basic_consume(
                self.publisher.response().queue().name(),
                "",
                *self.publisher.response().consume().options(),
                self.publisher.response().consume().arguments().clone(),
            ),
        )
        .await
        {
            Ok(result) => match result {
                Ok(consumer) => consumer,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::ApiError,
                        format!("failed to create consumer: {}", error),
                    ));
                }
            },
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("timed out creating consumer: {}", error),
                ));
            }
        };

        let delivery = match timeout(self.timeout_after, consumer.try_next()).await {
            Ok(result) => match result {
                Ok(delivery) => match delivery {
                    Some(delivery) => delivery,
                    None => {
                        return Err(Error::new(
                            ErrorKind::ApiError,
                            "received an empty delivery",
                        ));
                    }
                },
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::ApiError,
                        format!("failed to consume response: {}", error),
                    ));
                }
            },
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("timed out consuming response: {}", error),
                ));
            }
        };

        let value = match serde_json::from_slice::<Value>(delivery.data.as_slice()) {
            Ok(value) => value,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("failed to deserialize delivery: {}", error),
                ));
            }
        };

        Ok(value)
    }
}
