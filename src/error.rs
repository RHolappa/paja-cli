// SPDX-License-Identifier: Apache-2.0
use serde::Serialize;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub struct Error {
    pub code: &'static str,
    pub message: String,
    pub retryable: bool,
    pub retry_after_ms: Option<u64>,
    pub details: serde_json::Value,
    #[serde(skip)]
    pub exit: u8,
}

impl Error {
    pub fn new(code: &'static str, message: impl Into<String>, exit: u8) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: exit == 6,
            retry_after_ms: (exit == 6).then_some(100),
            details: serde_json::Value::Null,
            exit,
        }
    }
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new("invalid_input", message, 2)
    }
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new("revision_conflict", message, 4)
    }
    pub fn missing(message: impl Into<String>) -> Self {
        Self::new("required_input", message, 7)
    }
    pub fn integrity(message: impl Into<String>) -> Self {
        Self::new("integrity_error", message, 5)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::new("io_error", error.to_string(), 5)
    }
}
impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Self::new("storage_error", error.to_string(), 5)
    }
}
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::invalid(error.to_string())
    }
}
