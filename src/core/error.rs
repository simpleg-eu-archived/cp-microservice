use std::fmt;

use async_channel::{RecvError, SendError};
use serde::{Deserialize, Serialize};
use tokio::time::error::Elapsed;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorKind {
    ApiError,
    LogicError,
    StorageError,
    RequestError,
    Unknown,
    InitializationError,
    InternalError,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Error {
        Error {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<Elapsed> for Error {
    fn from(value: Elapsed) -> Self {
        Self::new(ErrorKind::InternalError, format!("timed out: {}", &value))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::new(ErrorKind::InternalError, format!("{}", &value))
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(value: SendError<T>) -> Self {
        Self::new(
            ErrorKind::InternalError,
            format!("failed to send request: {}", &value),
        )
    }
}

impl From<RecvError> for Error {
    fn from(value: RecvError) -> Self {
        Self::new(
            ErrorKind::InternalError,
            format!("failed to receive request: {}", &value),
        )
    }
}
