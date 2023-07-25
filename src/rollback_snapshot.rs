use serde::Serialize;
use std::fmt::{Display, Error, Formatter};

#[derive(Serialize)]
pub struct RollbackSnapshot<RollbackRequest: Serialize> {
    failure_message: String,
    pending_requests: Vec<RollbackRequest>,
}

impl<RollbackRequest: Serialize> RollbackSnapshot<RollbackRequest> {
    pub fn new(
        failure_message: String,
        pending_requests: Vec<RollbackRequest>,
    ) -> RollbackSnapshot<RollbackRequest> {
        RollbackSnapshot {
            failure_message,
            pending_requests,
        }
    }
}

impl<RollbackRequest: Serialize> Display for RollbackSnapshot<RollbackRequest> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(string) => {
                write!(f, "{}", string)
            }
            Err(error) => {
                log::warn!("failed to serialize RollbackSnapshot: {}", error);

                Err(Error::default())
            }
        }
    }
}

#[cfg(test)]
#[derive(Serialize)]
pub struct RollbackRequestDummy {
    id: String,
}

#[test]
pub fn display_expected_details_test() {
    const EXPECTED_MESSAGE: &str = "{\"failure_message\":\"example\",\"pending_requests\":[{\"id\":\"1\"},{\"id\":\"2\"},{\"id\":\"3\"}]}";
    let requests: Vec<RollbackRequestDummy> = vec![
        RollbackRequestDummy {
            id: "1".to_string(),
        },
        RollbackRequestDummy {
            id: "2".to_string(),
        },
        RollbackRequestDummy {
            id: "3".to_string(),
        },
    ];

    let snapshot = RollbackSnapshot::new("example".to_string(), requests);

    let result = format!("{}", snapshot);

    assert_eq!(EXPECTED_MESSAGE, result);
}
