use crate::api::server::input::executor::Executor;

pub struct Action<LogicRequestType> {
    id: String,
    executor: Executor<LogicRequestType>,
    filter_out_plugins: Vec<String>,
}

impl<LogicRequestType> Action<LogicRequestType> {
    pub fn new(
        id: String,
        executor: Executor<LogicRequestType>,
        filter_out_plugins: Vec<String>,
    ) -> Self {
        Self {
            id,
            executor,
            filter_out_plugins,
        }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn executor(&self) -> Executor<LogicRequestType> {
        self.executor.clone()
    }

    pub fn filter_out_plugins(&self) -> Vec<String> {
        self.filter_out_plugins.clone()
    }
}
