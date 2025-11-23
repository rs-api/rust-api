//! Foton - Fast, minimal, Rust-native web framework.
//!
//! ```rust,no_run
//! use foton::{Foton, Res};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut app = Foton::new();
//!     app.get("/", |_| async { Res::text("Hello") });
//!     app.listen(([127, 0, 0, 1], 3000)).await.unwrap();
//! }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod api;
mod config;
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

#[cfg(feature = "websocket")]
pub mod websocket;

pub use api::{Foton, app, app_with_state};
pub use config::ServerConfig;
pub use error::{Error, Result};
pub use error_handler::ErrorHandler;
pub use extensions::Extensions;
pub use extractors::{BodyBytes, Form, FromRequest, Headers, Json, Path, Query, State};
pub use handler::{FnHandler, FnHandler1, FnHandler2, FnHandler3, Handler};
pub use into_res::IntoRes;
pub use middleware::{Middleware, Next, from_fn, middleware};
pub use req::Req;
pub use res::{Res, ResBuilder, StreamSender};
pub use route::Route;
pub use router::Router;

#[cfg(feature = "websocket")]
pub use websocket::{CloseFrame, Message, WebSocket, WebSocketHandler, WebSocketUpgrade};

/// Common types and traits.
pub mod prelude {
    pub use crate::extractors::{BodyBytes, Form, FromRequest, Headers, Json, Path, Query, State};
    pub use crate::{
        Error, ErrorHandler, Extensions, Foton, Handler, IntoRes, Middleware, Next, Req, Res,
        Result, Route, Router, app, app_with_state, from_fn, middleware,
    };
}
