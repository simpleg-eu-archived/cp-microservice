use std::time::Duration;
use tokio::time::sleep;
use cp_microservice::process::PROCESS;
use cp_microservice::process_state::ProcessState;

#[tokio::main]
pub async fn main() {
    let state = PROCESS.state();

    let read_state = state.read().await;

    sleep(Duration::from_secs(1u64)).await;

    match *read_state {
        ProcessState::RUNNING => {
            std::process::exit(0);
        }
        ProcessState::STOPPING => std::process::exit(1)
    }
}