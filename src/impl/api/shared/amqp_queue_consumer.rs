use lapin::options::{BasicAckOptions, BasicRejectOptions};
use serde::{Deserialize, Serialize};

use crate::r#impl::api::shared::amqp_consume::AmqpConsume;
use crate::r#impl::api::shared::amqp_qos::AmqpQos;
use crate::r#impl::api::shared::amqp_queue::AmqpQueue;

#[derive(Deserialize, Serialize, Clone)]
pub struct AmqpQueueConsumer {
    queue: AmqpQueue,
    qos: AmqpQos,
    consume: AmqpConsume,
    acknowledge: BasicAckOptions,
    reject: BasicRejectOptions,
}

impl AmqpQueueConsumer {
    pub fn queue(&self) -> &AmqpQueue {
        &self.queue
    }

    pub fn qos(&self) -> &AmqpQos {
        &self.qos
    }

    pub fn consume(&self) -> &AmqpConsume {
        &self.consume
    }

    pub fn acknowledge(&self) -> &BasicAckOptions {
        &self.acknowledge
    }

    pub fn reject(&self) -> &BasicRejectOptions {
        &self.reject
    }
}
