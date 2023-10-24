use tokio::signal::unix::SignalKind;
use tokio_util::sync::CancellationToken;

pub fn listen_to_process_signals(cancellation_token: CancellationToken) {
    tokio::spawn(async move {
        let mut sigint = match tokio::signal::unix::signal(SignalKind::interrupt()) {
            Ok(sigint) => sigint,
            Err(error) => {
                panic!("failed to listen to SIGINT: {}", error);
            }
        };

        let mut sigterm = match tokio::signal::unix::signal(SignalKind::terminate()) {
            Ok(sigterm) => sigterm,
            Err(error) => {
                panic!("failed to listen to SIGTERM: {}", error);
            }
        };

        let mut sigquit = match tokio::signal::unix::signal(SignalKind::quit()) {
            Ok(sigquit) => sigquit,
            Err(error) => {
                panic!("failed to listen to SIGQUIT: {}", error);
            }
        };

        tokio::select! {
            _ = sigint.recv() => {
                cancellation_token.cancel();
            }
            _ = sigterm.recv() => {
                cancellation_token.cancel();
            }
            _ = sigquit.recv() => {
                cancellation_token.cancel();
            }
        }
    });
}
