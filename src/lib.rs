//! # Rust Api
//!
//! Fast and scalable web framework for Rust.
//!
//! ## Example
//!
//! ```rust,no_run
//! use rust_api::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = RustApi::new()
//!         .get("/", |_req: Req| async {
//!             Res::text("Hello, world!")
//!         })
//!         .get("/health", |_req: Req| async {
//!             Res::text("OK")
//!         });
//!
//!     app.listen(([127, 0, 0, 1], 3000)).await.unwrap();
//! }
//! ```
//!
//! ## Features
//!
//! - Fast async runtime
//! - Intuitive routing with nested routers
//! - Type-safe state management
//! - Composable middleware
//! - Zero-cost abstractions
//! - Configurable request body size limits

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod api;
mod error;
pub mod error_handler;
pub mod extensions;
pub mod extractors;
mod handler;
mod into_res;
pub mod layers;
mod middleware;
mod middleware_helpers;
mod req;
mod res;
mod router;

// Re-exports
pub use api::RustApi;
pub use error::{Error, Result};
pub use error_handler::{DefaultErrorHandler, ErrorHandler, FnErrorHandler, JsonErrorHandler};
pub use extensions::Extensions;
pub use extractors::{Form, FromRequest, Json, Path, Query, State};
pub use handler::{FnHandler, FnHandler1, FnHandler2, FnHandler3, Handler};
pub use into_res::IntoRes;
pub use middleware::{Middleware, Next};
pub use middleware_helpers::{CombinedMiddleware, ConditionalMiddleware, MiddlewareChain};
pub use req::Req;
pub use res::{Res, ResBuilder};
pub use router::Router;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::extractors::{Form, FromRequest, Json, Path, Query, State};
    pub use crate::{
        DefaultErrorHandler, Error, ErrorHandler, Extensions, FnErrorHandler, Handler, IntoRes,
        JsonErrorHandler, Middleware, Next, Req, Res, Result, Router, RustApi,
    };
}
