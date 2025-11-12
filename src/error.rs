//! Error handling types
//!
//! Provides HTTP-aware error types that automatically convert
//! to appropriate status codes and JSON responses.

use std::fmt;

/// Standard Result type for Rust Api
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during request handling
#[derive(Debug)]
pub enum Error {
    /// HTTP status code error with optional message
    Status(u16, Option<String>),

    /// JSON serialization/deserialization error
    Json(serde_json::Error),

    /// Hyper HTTP error
    Hyper(hyper::Error),

    /// IO error
    Io(std::io::Error),

    /// Custom error message
    Custom(String),
}

impl Error {
    /// Create a 400 Bad Request error
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::Status(400, Some(msg.into()))
    }

    /// Create a 401 Unauthorized error
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Status(401, Some(msg.into()))
    }

    /// Create a 403 Forbidden error
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Status(403, Some(msg.into()))
    }

    /// Create a 404 Not Found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::Status(404, Some(msg.into()))
    }

    /// Create a 422 Unprocessable Entity error
    pub fn unprocessable(msg: impl Into<String>) -> Self {
        Self::Status(422, Some(msg.into()))
    }

    /// Create a 500 Internal Server Error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Status(500, Some(msg.into()))
    }

    /// Create a custom status code error
    pub fn status(code: u16) -> Self {
        Self::Status(code, None)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Status(code, Some(msg)) => write!(f, "HTTP {}: {}", code, msg),
            Error::Status(code, None) => write!(f, "HTTP {}", code),
            Error::Json(e) => write!(f, "JSON error: {}", e),
            Error::Hyper(e) => write!(f, "HTTP error: {}", e),
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error::Hyper(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Custom(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::Custom(msg.to_string())
    }
}
