//! Convert types into HTTP responses
//!
//! The [`IntoRes`] trait allows handlers to return various types
//! that are automatically converted to HTTP responses.

use crate::{Error, Res};

/// Types that can become HTTP responses
pub trait IntoRes {
    /// Convert into response
    fn into_res(self) -> Res;
}

impl IntoRes for Res {
    fn into_res(self) -> Res {
        self
    }
}

impl IntoRes for String {
    fn into_res(self) -> Res {
        Res::text(self)
    }
}

impl IntoRes for &'static str {
    fn into_res(self) -> Res {
        Res::text(self)
    }
}

impl IntoRes for () {
    fn into_res(self) -> Res {
        Res::status(204) // No Content
    }
}

impl<T: IntoRes> IntoRes for Result<T, Error> {
    fn into_res(self) -> Res {
        match self {
            Ok(value) => value.into_res(),
            Err(err) => err.into_res(),
        }
    }
}

impl IntoRes for Error {
    fn into_res(self) -> Res {
        // Use DefaultErrorHandler for now
        // The actual error handler will be applied in the handler execution
        use crate::error_handler::{DefaultErrorHandler, ErrorHandler};
        DefaultErrorHandler.handle(self)
    }
}

/// Wrapper for HTML responses
pub struct Html(pub String);

impl IntoRes for Html {
    fn into_res(self) -> Res {
        Res::html(self.0)
    }
}

impl IntoRes for &Html {
    fn into_res(self) -> Res {
        Res::html(self.0.clone())
    }
}
