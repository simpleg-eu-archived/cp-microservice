use std::time::Duration;

use tokio::time::sleep;

use cp_microservice::process::PROCESS;
use cp_microservice::process_request::ProcessRequest;

/// This test requires to monitor the process elapsed time in order to make sure it runs for
/// at least 5 seconds.
/// Also, it is expected to exit with code 1.
#[tokio::main]
pub async fn main() {
    tokio::spawn(async {
        sleep(Duration::from_secs(5)).await;
    });

    tokio::spawn(async {
        sleep(Duration::from_secs(3)).await;
    });

    PROCESS.sender().send(ProcessRequest::STOP).await.unwrap();

    sleep(Duration::from_secs(8)).await;
}
