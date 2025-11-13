//! Middleware composition helpers for Rust API framework.
//!
//! ## Available Helpers
//!
//! - `When` - Execute middleware based on predicate

use async_trait::async_trait;
use rust_api::{Middleware, Next, Req, Res};
use std::sync::Arc;

/// Execute middleware only when predicate returns true.
///
/// ## Example
///
/// ```rust
/// use rust_api_helpers::When;
/// use rust_api::{from_fn, Req, Res, Next};
/// use std::sync::Arc;
///
/// let logging = from_fn(|req: Req, _state: Arc<()>, next: Next| async move {
///     println!("Logging: {} {}", req.method(), req.uri());
///     next.run(req).await
/// });
///
/// // Only log API requests
/// let api_logging = When::new(
///     logging,
///     |req, _state| req.uri().path().starts_with("/api")
/// );
/// ```
pub struct When<S, M, F> {
    middleware: M,
    predicate: F,
    _marker: std::marker::PhantomData<S>,
}

impl<S, M, F> When<S, M, F> {
    /// Create middleware that executes conditionally.
    ///
    /// The predicate receives a reference to the request and state,
    /// returning true if the middleware should execute.
    pub fn new(middleware: M, predicate: F) -> Self {
        Self {
            middleware,
            predicate,
            _marker: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<S, M, F> Middleware<S> for When<S, M, F>
where
    S: Send + Sync + 'static,
    M: Middleware<S>,
    F: Fn(&Req, &Arc<S>) -> bool + Send + Sync + 'static,
{
    async fn handle(&self, req: Req, state: Arc<S>, next: Next<S>) -> Res {
        if (self.predicate)(&req, &state) {
            self.middleware.handle(req, state, next).await
        } else {
            next.run(req).await
        }
    }
}
