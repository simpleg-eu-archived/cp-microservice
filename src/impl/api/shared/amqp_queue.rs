use serde::{Deserialize, Serialize};

use crate::r#impl::api::shared::amqp_queue_declare::AmqpQueueDeclare;

#[derive(Deserialize, Serialize, Clone)]
pub struct AmqpQueue {
    name: String,
    declare: AmqpQueueDeclare,
}

impl AmqpQueue {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn declare(&self) -> &AmqpQueueDeclare {
        &self.declare
    }
}
