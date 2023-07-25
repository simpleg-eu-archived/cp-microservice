use crate::rollback_snapshot::RollbackSnapshot;
use async_channel::Sender;
use serde::Serialize;
use std::time::Duration;
use tokio::time::timeout;

///
/// Stack of rollback requests which will be sent through a specific channel
/// if a multi-step operation fails.
///
/// The request must be Serializable so detailed logging can be provided
/// if the rollback fails.
///
pub struct RollbackStack<RollbackRequest: Serialize> {
    requests: Vec<RollbackRequest>,
    sender: Sender<RollbackRequest>,
}

impl<RollbackRequest: Serialize> RollbackStack<RollbackRequest> {
    pub fn new(sender: Sender<RollbackRequest>) -> RollbackStack<RollbackRequest> {
        RollbackStack {
            requests: Vec::new(),
            sender,
        }
    }

    pub fn push(&mut self, request: RollbackRequest) {
        self.requests.push(request)
    }

    pub async fn rollback(mut self) -> Result<(), RollbackSnapshot<RollbackRequest>> {
        loop {
            match self.requests.pop() {
                Some(request) => match self.sender.send(request).await {
                    Ok(_) => (),
                    Err(error) => {
                        return Err(RollbackSnapshot::new(
                            format!("failed to send rollback request: {}", error),
                            self.requests,
                        ));
                    }
                },
                None => return Ok(()),
            }
        }
    }
}

#[cfg(test)]

const DEFAULT_TIMEOUT_MILLISECONDS: u64 = 100u64;

#[derive(Serialize)]
pub struct RollbackRequestDummy {
    id: i32,
}

#[tokio::test]
pub async fn execute_rollback_in_expected_order_test() {
    let (sender, receiver) = async_channel::unbounded::<RollbackRequestDummy>();
    let mut rollback_stack: RollbackStack<RollbackRequestDummy> = RollbackStack::new(sender);

    let request_1 = RollbackRequestDummy { id: 1 };
    let request_2 = RollbackRequestDummy { id: 2 };
    let request_3 = RollbackRequestDummy { id: 3 };

    rollback_stack.push(request_1);
    rollback_stack.push(request_2);
    rollback_stack.push(request_3);

    match timeout(
        Duration::from_secs(DEFAULT_TIMEOUT_MILLISECONDS),
        rollback_stack.rollback(),
    )
    .await
    {
        Ok(_) => (),
        Err(_) => {
            panic!("default timeout time elapsed for completing rollback")
        }
    }

    for expected_request_id in (1..4).rev() {
        match timeout(
            Duration::from_millis(DEFAULT_TIMEOUT_MILLISECONDS),
            receiver.recv(),
        )
        .await
        {
            Ok(result) => match result {
                Ok(request) => {
                    assert_eq!(expected_request_id, request.id)
                }
                Err(_) => {
                    panic!("failed to receive rollback request")
                }
            },
            Err(_) => {
                panic!("default timeout time elapsed for receiving rollback request")
            }
        }
    }
}

#[tokio::test]
pub async fn return_error_when_channel_closes_test() {
    let (sender, _receiver) = async_channel::unbounded::<RollbackRequestDummy>();
    sender.close();

    let mut rollback_stack: RollbackStack<RollbackRequestDummy> = RollbackStack::new(sender);
    rollback_stack.push(RollbackRequestDummy { id: 1 });

    if (rollback_stack.rollback().await).is_ok() {
        panic!("expected rollback to fail")
    }
}
