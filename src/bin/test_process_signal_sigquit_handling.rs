use cp_microservice::r#impl::process_signals::listen_to_process_signals;
use std::process::Command;
use tokio_util::sync::CancellationToken;

#[tokio::main]
pub async fn main() {
    let cancellation_token = CancellationToken::new();

    let pid = std::process::id();

    listen_to_process_signals(cancellation_token.clone());

    Command::new("kill")
        .arg("-SIGQUIT")
        .arg(pid.to_string())
        .spawn()
        .expect("failed to send SIGQUIT to process")
        .wait()
        .expect("failed to await for SIGQUIT to be sent to process");

    assert!(cancellation_token.is_cancelled());
}
