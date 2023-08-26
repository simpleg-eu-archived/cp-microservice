use crate::api::server::input::replier::Replier;
use crate::api::shared::request::Request;

pub struct InputData {
    pub request: Request,
    pub replier: Replier,
}

impl InputData {
    pub fn new(request: Request, replier: Replier) -> InputData {
        InputData { request, replier }
    }
}
