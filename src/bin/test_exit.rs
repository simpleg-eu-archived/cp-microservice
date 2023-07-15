use cp_microservice::process::PROCESS;
use cp_microservice::process_request::ProcessRequest;

#[tokio::main]
pub async fn main() {
    PROCESS.sender().send(ProcessRequest::STOP).await;

    loop {

    }
}