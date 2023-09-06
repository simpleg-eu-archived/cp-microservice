use lapin::options::BasicQosOptions;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct AmqpQos {
    prefetch_count: u16,
    options: BasicQosOptions,
}

impl AmqpQos {
    pub fn prefetch_count(&self) -> u16 {
        self.prefetch_count
    }

    pub fn options(&self) -> &BasicQosOptions {
        &self.options
    }
}
