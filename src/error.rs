//! Error types.

use std::fmt;

/// Result type with framework Error.
pub type Result<T> = std::result::Result<T, Error>;

/// HTTP error.
#[derive(Debug)]
pub enum Error {
    /// HTTP status with optional message.
    Status(u16, Option<String>),
    /// JSON error.
    Json(String),
    /// HTTP protocol error.
    Hyper(hyper::Error),
    /// IO error.
    Io(std::io::Error),
    /// Custom error.
    Custom(String),
}

impl Error {
    /// Create 400 Bad Request.
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::Status(400, Some(msg.into()))
    }

    /// Create 401 Unauthorized.
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Status(401, Some(msg.into()))
    }

    /// Create 403 Forbidden.
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Status(403, Some(msg.into()))
    }

    /// Create 404 Not Found.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::Status(404, Some(msg.into()))
    }

    /// Create 405 Method Not Allowed.
    pub fn method_not_allowed(msg: impl Into<String>) -> Self {
        Self::Status(405, Some(msg.into()))
    }

    /// Create 413 Payload Too Large.
    pub fn payload_too_large(msg: impl Into<String>) -> Self {
        Self::Status(413, Some(msg.into()))
    }

    /// Create 422 Unprocessable Entity.
    pub fn unprocessable(msg: impl Into<String>) -> Self {
        Self::Status(422, Some(msg.into()))
    }

    /// Create 500 Internal Server Error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Status(500, Some(msg.into()))
    }

    /// Create custom status code.
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
