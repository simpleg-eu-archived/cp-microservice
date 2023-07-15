use crate::process_request::ProcessRequest;
use crate::process_state::ProcessState;
use log::{error, warn};
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;
use tokio::sync::{RwLock};
use tokio::time::timeout;

pub static PROCESS: Lazy<Process> = Lazy::new(|| {
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<ProcessRequest>(1024usize);
    let process = Process {
        state: Arc::new(RwLock::new(ProcessState::RUNNING)),
        sender,
    };

    let state = process.state();

    tokio::spawn(async move {
        let metrics = Handle::current().metrics();

        loop {
            match timeout(Duration::from_millis(100), receiver.recv()).await {
                Ok(request) => match request {
                    Some(request) => match request {
                        ProcessRequest::STOP => {
                            warn!("process stopping");

                            let mut state_write_guard = state.write().await;

                            *state_write_guard = ProcessState::STOPPING;
                        }
                    },
                    None => {
                        error!("process mpsc channel has been closed");
                    }
                }
                Err(_) => {}
            }

            let state_read_guard = state.read().await;

            match *state_read_guard {
                ProcessState::RUNNING => (),
                ProcessState::STOPPING => {
                    // All tasks have finished, so we are the only one remaining.
                    if metrics.active_tasks_count() == 1 {
                        std::process::exit(1);
                    }
                }
            }
        }
    });

    process
});

pub struct Process {
    state: Arc<RwLock<ProcessState>>,
    sender: Sender<ProcessRequest>,
}

impl Process {
    pub fn state(&self) -> Arc<RwLock<ProcessState>> {
        self.state.clone()
    }

    pub fn sender(&self) -> Sender<ProcessRequest> {
        self.sender.clone()
    }
}
