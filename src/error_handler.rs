//! Custom error handlers
//!
//! Allows applications to define how errors are converted into HTTP responses.

use crate::{Error, Res};

/// Trait for converting errors into HTTP responses
///
/// Implement this trait to customize how your application handles errors.
///
/// # Example
///
/// ```rust,ignore
/// use rust_api::prelude::*;
///
/// struct JsonErrorHandler;
///
/// impl ErrorHandler for JsonErrorHandler {
///     fn handle(&self, error: Error) -> Res {
///         let json = format!(
///             r#"{{"error": "{}", "code": {}}}"#,
///             error,
///             status_code(&error)
///         );
///         Res::builder()
///             .status(status_code(&error))
///             .header("Content-Type", "application/json")
///             .text(json)
///     }
/// }
///
/// let app = RustApi::new()
///     .error_handler(JsonErrorHandler)
///     .get("/", handler);
/// ```
pub trait ErrorHandler: Send + Sync + 'static {
    /// Convert an error into an HTTP response
    fn handle(&self, error: Error) -> Res;
}

/// Default error handler that provides plain text responses
#[derive(Debug, Clone, Copy)]
pub struct DefaultErrorHandler;

impl ErrorHandler for DefaultErrorHandler {
    fn handle(&self, error: Error) -> Res {
        match error {
            Error::Status(code, Some(msg)) => Res::builder()
                .status(code)
                .text(format!("{} {}", code, msg)),
            Error::Status(code, None) => Res::status(code),
            Error::Json(e) => Res::builder()
                .status(400)
                .text(format!("JSON error: {}", e)),
            Error::Hyper(e) => Res::builder()
                .status(500)
                .text(format!("HTTP error: {}", e)),
            Error::Io(e) => Res::builder().status(500).text(format!("IO error: {}", e)),
            Error::Custom(msg) => Res::builder().status(500).text(msg),
        }
    }
}

/// JSON error handler that returns errors as JSON
#[derive(Debug, Clone, Copy)]
pub struct JsonErrorHandler;

impl ErrorHandler for JsonErrorHandler {
    fn handle(&self, error: Error) -> Res {
        let (status_code, message) = match &error {
            Error::Status(code, Some(msg)) => (*code, msg.clone()),
            Error::Status(code, None) => (*code, status_text(*code)),
            Error::Json(e) => (400, format!("JSON error: {}", e)),
            Error::Hyper(e) => (500, format!("HTTP error: {}", e)),
            Error::Io(e) => (500, format!("IO error: {}", e)),
            Error::Custom(msg) => (500, msg.clone()),
        };

        let json = format!(
            r#"{{"error":"{}","status":{}}}"#,
            escape_json(&message),
            status_code
        );

        Res::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .text(json)
    }
}

/// Function-based error handler
pub struct FnErrorHandler<F>(pub F);

impl<F> ErrorHandler for FnErrorHandler<F>
where
    F: Fn(Error) -> Res + Send + Sync + 'static,
{
    fn handle(&self, error: Error) -> Res {
        (self.0)(error)
    }
}

// Helper functions

fn status_text(code: u16) -> String {
    match code {
        400 => "Bad Request".to_string(),
        401 => "Unauthorized".to_string(),
        403 => "Forbidden".to_string(),
        404 => "Not Found".to_string(),
        405 => "Method Not Allowed".to_string(),
        413 => "Payload Too Large".to_string(),
        422 => "Unprocessable Entity".to_string(),
        500 => "Internal Server Error".to_string(),
        502 => "Bad Gateway".to_string(),
        503 => "Service Unavailable".to_string(),
        _ => format!("HTTP {}", code),
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
    }
}
