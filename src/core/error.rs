use std::fmt;

use serde::Serialize;

#[derive(Debug, Copy, Clone, PartialEq, Serialize)]
pub enum ErrorKind {
    ApiError,
    LogicError,
    StorageError,
    RequestError,
    Unknown,
    InitializationError,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
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
