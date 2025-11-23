//! Trait-based middleware.

use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;

use crate::{Req, Res};

/// Middleware trait for request interception.
#[async_trait]
pub trait Middleware<S = ()>: Send + Sync + 'static {
    /// Handle request before passing to next middleware/handler.
    async fn handle(&self, req: Req, state: Arc<S>, next: Next<S>) -> Res;
}

/// Next middleware/handler in chain.
pub struct Next<S = ()> {
    pub(crate) handler: Arc<dyn Fn(Req, Arc<S>) -> BoxFuture<Res> + Send + Sync>,
    pub(crate) state: Arc<S>,
}

type BoxFuture<T> = std::pin::Pin<Box<dyn Future<Output = T> + Send>>;

impl<S: 'static> Next<S> {
    /// Create next handler.
    #[inline]
    pub fn new(
        handler: Arc<dyn Fn(Req, Arc<S>) -> BoxFuture<Res> + Send + Sync>,
        state: Arc<S>,
    ) -> Self {
        Self { handler, state }
    }

    /// Run next handler.
    #[inline]
    pub async fn run(self, req: Req) -> Res {
        (self.handler)(req, self.state).await
    }
}

/// Function-based middleware wrapper.
pub struct FnMiddleware<F>(pub F);

#[async_trait]
impl<F, Fut, S> Middleware<S> for FnMiddleware<F>
where
    F: Fn(Req, Arc<S>, Next<S>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    S: Send + Sync + 'static,
{
    async fn handle(&self, req: Req, state: Arc<S>, next: Next<S>) -> Res {
        (self.0)(req, state, next).await
    }
}

/// Create middleware from function.
///
/// ```rust
/// use foton::{from_fn, Req, Res, Next};
/// use std::sync::Arc;
///
/// let logging = from_fn(|req: Req, _state: Arc<()>, next: Next<()>| async move {
///     println!("{} {}", req.method(), req.uri());
///     next.run(req).await
/// });
/// ```
pub fn from_fn<F, Fut, S>(f: F) -> FnMiddleware<F>
where
    F: Fn(Req, Arc<S>, Next<S>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    S: Send + Sync + 'static,
{
    FnMiddleware(f)
}

/// Alias for `from_fn`.
pub fn middleware<F, Fut, S>(f: F) -> FnMiddleware<F>
where
    F: Fn(Req, Arc<S>, Next<S>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send + 'static,
    S: Send + Sync + 'static,
{
    from_fn(f)
}
