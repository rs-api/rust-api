//! Per-route configuration with middleware support.

use hyper::Method;
use std::sync::Arc;

use crate::{Handler, Middleware, handler::IntoHandler};

/// Route with per-route middleware.
pub struct Route<S = ()> {
    pub(crate) method: Method,
    pub(crate) path: String,
    pub(crate) handler: Arc<dyn Handler<S>>,
    pub(crate) middlewares: Arc<Vec<Arc<dyn Middleware<S>>>>,
}

impl<S: Send + Sync + 'static> Route<S> {
    pub(crate) fn new(method: Method, path: String, handler: Arc<dyn Handler<S>>) -> Self {
        Self {
            method,
            path,
            handler,
            middlewares: Arc::new(Vec::new()),
        }
    }

    /// Add middleware to this route.
    ///
    /// Middleware is executed in registration order.
    pub fn layer<M: Middleware<S>>(&mut self, middleware: M) {
        let mut mw = (*self.middlewares).clone();
        mw.push(Arc::new(middleware));
        self.middlewares = Arc::new(mw);
    }

    /// Create a GET route.
    pub fn get<H, T>(path: impl Into<String>, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        Self::new(Method::GET, path.into(), handler.into_handler())
    }

    /// Create a POST route.
    pub fn post<H, T>(path: impl Into<String>, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        Self::new(Method::POST, path.into(), handler.into_handler())
    }

    /// Create a PUT route.
    pub fn put<H, T>(path: impl Into<String>, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        Self::new(Method::PUT, path.into(), handler.into_handler())
    }

    /// Create a DELETE route.
    pub fn delete<H, T>(path: impl Into<String>, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        Self::new(Method::DELETE, path.into(), handler.into_handler())
    }

    /// Create a PATCH route.
    pub fn patch<H, T>(path: impl Into<String>, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        Self::new(Method::PATCH, path.into(), handler.into_handler())
    }
}
