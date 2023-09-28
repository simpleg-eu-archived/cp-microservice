use serde::{Deserialize, Serialize};

use crate::r#impl::api::shared::amqp_queue_consumer::AmqpQueueConsumer;

#[derive(Deserialize, Serialize)]
pub struct AmqpApiEntry {
    pub amqp_queue_consumer: AmqpQueueConsumer,
}
