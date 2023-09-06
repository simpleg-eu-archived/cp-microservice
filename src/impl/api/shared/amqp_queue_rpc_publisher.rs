use serde::{Deserialize, Serialize};

use crate::r#impl::api::shared::amqp_publish::AmqpPublish;
use crate::r#impl::api::shared::amqp_queue_consumer::AmqpQueueConsumer;

#[derive(Deserialize, Serialize, Clone)]
pub struct AmqpQueueRpcPublisher {
    queue_name: String,
    publish: AmqpPublish,
    response: AmqpQueueConsumer,
}

impl AmqpQueueRpcPublisher {
    pub fn queue_name(&self) -> &str {
        self.queue_name.as_str()
    }

    pub fn publish(&self) -> &AmqpPublish {
        &self.publish
    }

    pub fn response(&self) -> &AmqpQueueConsumer {
        &self.response
    }
}
