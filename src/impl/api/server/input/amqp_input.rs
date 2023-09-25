use std::sync::Arc;

use async_trait::async_trait;
use futures_util::TryStreamExt;
use lapin::message::Delivery;
use lapin::options::{BasicAckOptions, BasicPublishOptions, BasicRejectOptions};
use lapin::types::ShortString;
use lapin::{BasicProperties, Channel, Consumer};
use uuid::Uuid;

use crate::api::server::input::input::Input;
use crate::api::server::input::input_data::InputData;
use crate::api::server::input::replier::Replier;
use crate::api::shared::request::Request;
use crate::core::error::{Error, ErrorKind};
use crate::r#impl::api::shared::amqp_queue_consumer::AmqpQueueConsumer;

pub struct AmqpInput<'a> {
    channel: Arc<Channel>,
    consumer: Consumer,
    reject_options: BasicRejectOptions,
    ack_options: BasicAckOptions,
    filter_out_plugins: Vec<&'a str>,
}

impl<'a> AmqpInput<'a> {
    pub async fn try_new(
        channel: Arc<Channel>,
        queue_consumer: AmqpQueueConsumer,
        filter_out_plugins: Vec<&'a str>,
    ) -> Result<AmqpInput, Error> {
        let _queue = match channel
            .queue_declare(
                queue_consumer.queue().name(),
                *queue_consumer.queue().declare().options(),
                queue_consumer.queue().declare().arguments().clone(),
            )
            .await
        {
            Ok(queue) => queue,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("failed to declare queue: {}", error),
                ));
            }
        };

        match channel
            .basic_qos(
                queue_consumer.qos().prefetch_count(),
                *queue_consumer.qos().options(),
            )
            .await
        {
            Ok(()) => (),
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("failure basic qos: {}", error),
                ));
            }
        }

        let consumer = match AmqpInput::try_get_consumer(&channel, &queue_consumer).await {
            Ok(consumer) => consumer,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("failed to create consumer: {}", error),
                ));
            }
        };

        let reject_options = *queue_consumer.reject();
        let ack_options = *queue_consumer.acknowledge();

        Ok(Self {
            channel,
            consumer,
            reject_options,
            ack_options,
            filter_out_plugins,
        })
    }

    async fn try_get_consumer(
        channel: &Arc<Channel>,
        queue_consumer: &AmqpQueueConsumer,
    ) -> Result<Consumer, Error> {
        let consumer_tag = format!("{}#{}", queue_consumer.queue().name(), Uuid::new_v4());
        let consumer = match channel
            .basic_consume(
                queue_consumer.queue().name(),
                consumer_tag.as_str(),
                *queue_consumer.consume().options(),
                queue_consumer.consume().arguments().clone(),
            )
            .await
        {
            Ok(consumer) => consumer,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("failure basic consume: {}", error),
                ));
            }
        };

        Ok(consumer)
    }

    async fn reject_delivery(&self, delivery: Delivery, rejection_error: Error) -> Error {
        match delivery.reject(self.reject_options).await {
            Ok(_) => rejection_error,
            Err(error) => Error::new(
                ErrorKind::ApiError,
                format!("failed to reject delivery: {}", error),
            ),
        }
    }
}

#[async_trait]
impl<'a> Input for AmqpInput<'a> {
    async fn receive(&mut self) -> Result<InputData, Error> {
        let delivery = match self.consumer.try_next().await {
            Ok(optional_delivery) => match optional_delivery {
                Some(delivery) => delivery,
                None => {
                    return Err(Error::new(
                        ErrorKind::ApiError,
                        "consumer got an empty delivery",
                    ));
                }
            },
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    format!("consumer got an error: {}", error),
                ));
            }
        };

        let json_request = match std::str::from_utf8(delivery.data.as_slice()) {
            Ok(json_request) => json_request,
            Err(error) => {
                return Err(self
                    .reject_delivery(
                        delivery,
                        Error::new(
                            ErrorKind::RequestError,
                            format!("delivery is not an utf8 string: {}", error),
                        ),
                    )
                    .await);
            }
        };

        let request: Request = match serde_json::from_str(json_request) {
            Ok(request) => request,
            Err(error) => {
                return Err(self
                    .reject_delivery(
                        delivery,
                        Error::new(
                            ErrorKind::RequestError,
                            format!("failed to deserialize request: {}", error),
                        ),
                    )
                    .await);
            }
        };

        if let Err(error) = delivery.ack(self.ack_options).await {
            log::warn!("failed to acknowledge delivery: {}", error);
        }

        let channel = self.channel.clone();
        let properties: BasicProperties = delivery.properties;

        let replier: Replier = Arc::new(move |value| {
            let channel = channel.clone();
            let properties = properties.clone();

            Box::pin(async move {
                let request_properties = properties;

                let reply_to = match request_properties.reply_to() {
                    Some(reply_to) => reply_to,
                    None => return Ok(()),
                };

                let mut response_properties = BasicProperties::default()
                    .with_content_type(ShortString::from("application/json"));

                if let Some(correlation_id) = request_properties.correlation_id() {
                    response_properties =
                        response_properties.with_correlation_id(correlation_id.clone());
                }

                let publish_options = BasicPublishOptions::default();
                let payload = match serde_json::to_vec(&value) {
                    Ok(payload) => payload,
                    Err(error) => {
                        return Err(Error::new(
                            ErrorKind::ApiError,
                            format!("failed to serialize result: {}", error),
                        ));
                    }
                };

                match channel
                    .basic_publish(
                        "",
                        reply_to.as_str(),
                        publish_options,
                        payload.as_slice(),
                        response_properties,
                    )
                    .await
                {
                    Ok(_) => (),
                    Err(error) => {
                        return Err(Error::new(
                            ErrorKind::ApiError,
                            format!("failed to send reply: {}", error),
                        ));
                    }
                }

                Ok(())
            })
        });

        Ok(InputData::new(request, replier))
    }

    fn filter_out_plugins(&self) -> &[&str] {
        self.filter_out_plugins.as_slice()
    }
}
