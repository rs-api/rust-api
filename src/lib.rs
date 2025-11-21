//! Web framework for Rust.
//!
//! ```rust,no_run
//! use rust_api::{RustApi, Res};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut app = RustApi::new();
//!     app.get("/", |_| async { Res::text("Hello") });
//!     app.listen(([127, 0, 0, 1], 3000)).await.unwrap();
//! }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod api;
mod error;
pub mod error_handler;
pub mod extensions;
pub mod extractors;
mod handler;
mod into_res;
mod middleware;
mod req;
mod res;
pub mod route;
mod router;

pub use api::RustApi;
pub use error::{Error, Result};
pub use error_handler::ErrorHandler;
pub use extensions::Extensions;
pub use extractors::{BodyBytes, Form, FromRequest, Headers, Json, Path, Query, State};
pub use handler::{FnHandler, FnHandler1, FnHandler2, FnHandler3, Handler};
pub use into_res::IntoRes;
pub use middleware::{Middleware, Next, from_fn, middleware};
pub use req::Req;
pub use res::{Res, ResBuilder};
pub use route::Route;
pub use router::Router;

/// Common types and traits.
pub mod prelude {
    pub use crate::extractors::{BodyBytes, Form, FromRequest, Headers, Json, Path, Query, State};
    pub use crate::{
        Error, ErrorHandler, Extensions, Handler, IntoRes, Middleware, Next, Req, Res, Result,
        Route, Router, RustApi, from_fn, middleware,
    };
}
