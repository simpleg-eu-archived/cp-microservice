use lapin::options::BasicPublishOptions;
use lapin::protocol::basic::AMQPProperties;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct AmqpPublish {
    exchange: String,
    options: BasicPublishOptions,
    properties: AMQPProperties,
}

impl AmqpPublish {
    pub fn exchange(&self) -> &str {
        &self.exchange
    }

    pub fn options(&self) -> &BasicPublishOptions {
        &self.options
    }

    pub fn properties(&self) -> &AMQPProperties {
        &self.properties
    }
}
