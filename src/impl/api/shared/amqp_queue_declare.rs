use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct AmqpQueueDeclare {
    pub options: QueueDeclareOptions,
    pub arguments: FieldTable,
}

impl AmqpQueueDeclare {
    pub fn options(&self) -> &QueueDeclareOptions {
        &self.options
    }

    pub fn arguments(&self) -> &FieldTable {
        &self.arguments
    }
}
