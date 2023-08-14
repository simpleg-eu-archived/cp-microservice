use serde::Serialize;
use std::fmt::{Display, Error, Formatter};

///
/// Facility struct whose main objective is to provide a detailed status when
/// when the rollback operation has failed.
///
#[derive(Serialize)]
pub struct RollbackSnapshot<RollbackRequestType: Serialize> {
    failure_message: String,
    pending_requests: Vec<RollbackRequestType>,
}

impl<RollbackRequestType: Serialize> RollbackSnapshot<RollbackRequestType> {
    pub fn new(
        failure_message: String,
        pending_requests: Vec<RollbackRequestType>,
    ) -> RollbackSnapshot<RollbackRequestType> {
        RollbackSnapshot {
            failure_message,
            pending_requests,
        }
    }
}

impl<RollbackRequestType: Serialize> Display for RollbackSnapshot<RollbackRequestType> {
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
