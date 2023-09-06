use lapin::options::BasicConsumeOptions;
use lapin::types::FieldTable;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct AmqpConsume {
    options: BasicConsumeOptions,
    arguments: FieldTable,
}

impl AmqpConsume {
    pub fn options(&self) -> &BasicConsumeOptions {
        &self.options
    }

    pub fn arguments(&self) -> &FieldTable {
        &self.arguments
    }
}
