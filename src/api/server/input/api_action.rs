use std::{fmt::Display, time::Duration};

use async_channel::Sender;
use serde::Serialize;
use serde_json::Value;
use tokio::{sync::oneshot::Receiver, time::timeout};

use crate::core::error::{Error, ErrorKind};

pub async fn api_action<OkResultType: Serialize, ErrResultType: Display, LogicRequestType>(
    logic_request: LogicRequestType,
    logic_request_sender: Sender<LogicRequestType>,
    timeout_after_milliseconds: u64,
    receiver: Receiver<Result<OkResultType, ErrResultType>>,
) -> Result<Value, Error> {
    match logic_request_sender.send(logic_request).await {
        Ok(_) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::ApiError,
                format!("failed to send logic request: {}", &error),
            ))
        }
    }

    let ok_result: OkResultType =
        match timeout(Duration::from_millis(timeout_after_milliseconds), receiver).await {
            Ok(result) => match result {
                Ok(result) => match result {
                    Ok(ok_result) => ok_result,
                    Err(error) => {
                        return Err(Error::new(
                            ErrorKind::RequestError,
                            format!("failed to handle request: {}", &error),
                        ))
                    }
                },
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::RequestError,
                        format!("failed to receive logic request: {}", &error),
                    ))
                }
            },
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::ApiError,
                    "timed out waiting for logic result",
                ))
            }
        };

    let serialized_ok_result = match serde_json::to_value(ok_result) {
        Ok(serialized_ok_result) => serialized_ok_result,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::ApiError,
                format!("failed to serialize result: {}", &error),
            ))
        }
    };

    Ok(serialized_ok_result)
}
