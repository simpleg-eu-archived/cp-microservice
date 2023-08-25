use cp_microservice::process::PROCESS;
use cp_microservice::process_request::ProcessRequest;

/// Expected to exit with code 1.
#[tokio::main]
pub async fn main() {
    PROCESS.sender().send(ProcessRequest::STOP).await.unwrap();

    loop {}
}
