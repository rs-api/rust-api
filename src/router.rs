//! Router for grouping routes with shared middleware.

use hyper::Method;
use std::sync::Arc;

use crate::{Handler, Middleware, handler::IntoHandler};

type BoxedHandler<S> = Arc<dyn Handler<S>>;
type BoxedMiddleware<S> = Arc<dyn Middleware<S>>;
type SharedMiddlewares<S> = Arc<Vec<BoxedMiddleware<S>>>;

/// Router for grouping routes with shared middleware.
pub struct Router<S = ()> {
    routes: Vec<(Method, String, BoxedHandler<S>)>,
    middlewares: Vec<BoxedMiddleware<S>>,
    nested: Vec<(String, Router<S>)>,
}

impl<S: Send + Sync + 'static> Router<S> {
    /// Create router with pre-allocated capacity.
    pub fn with_capacity(routes: usize, middlewares: usize) -> Self {
        Self {
            routes: Vec::with_capacity(routes),
            middlewares: Vec::with_capacity(middlewares),
            nested: Vec::new(),
        }
    }

    /// Create a new router.
    pub fn new() -> Self {
        Self::with_capacity(10, 5)
    }

    /// Register a GET route.
    pub fn get<H, T>(&mut self, path: &str, handler: H)
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::GET, path.to_string(), handler.into_handler()));
    }

    /// Register a POST route.
    pub fn post<H, T>(&mut self, path: &str, handler: H)
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::POST, path.to_string(), handler.into_handler()));
    }

    /// Register a PUT route.
    pub fn put<H, T>(&mut self, path: &str, handler: H)
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::PUT, path.to_string(), handler.into_handler()));
    }

    /// Register a DELETE route.
    pub fn delete<H, T>(&mut self, path: &str, handler: H)
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::DELETE, path.to_string(), handler.into_handler()));
    }

    /// Register a PATCH route.
    pub fn patch<H, T>(&mut self, path: &str, handler: H)
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::PATCH, path.to_string(), handler.into_handler()));
    }

    /// Add middleware to this router.
    ///
    /// Middleware applies to all routes in this router, including nested routers.
    pub fn layer<M: Middleware<S>>(&mut self, middleware: M) {
        self.middlewares.push(Arc::new(middleware));
    }

    /// Mount a nested router at a prefix.
    ///
    /// Middleware from parent router is inherited by nested router.
    pub fn nest(&mut self, prefix: &str, router: Router<S>) {
        self.nested.push((prefix.to_string(), router));
    }

    /// Get the number of routes in this router (excluding nested).
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    pub(crate) fn flatten(
        self,
        prefix: &str,
    ) -> Vec<(Method, String, BoxedHandler<S>, SharedMiddlewares<S>)> {
        self.flatten_with_shared("", prefix, None)
    }

    fn flatten_with_shared(
        self,
        base_prefix: &str,
        prefix: &str,
        parent_middlewares: Option<&SharedMiddlewares<S>>,
    ) -> Vec<(Method, String, BoxedHandler<S>, SharedMiddlewares<S>)> {
        let estimated_size = self.routes.len()
            + self
                .nested
                .iter()
                .map(|(_, r)| r.routes.len())
                .sum::<usize>();
        let mut flattened = Vec::with_capacity(estimated_size);

        let combined_middlewares: SharedMiddlewares<S> = if let Some(parent) = parent_middlewares {
            if self.middlewares.is_empty() {
                Arc::clone(parent)
            } else {
                let mut combined = Vec::with_capacity(parent.len() + self.middlewares.len());
                combined.extend_from_slice(parent);
                combined.extend_from_slice(&self.middlewares);
                Arc::new(combined)
            }
        } else {
            Arc::new(self.middlewares.clone())
        };

        for (method, path, handler) in self.routes {
            let full_path = if prefix.is_empty() {
                path.clone()
            } else {
                format!("{}{}", prefix, path)
            };

            flattened.push((
                method.clone(),
                full_path,
                Arc::clone(&handler),
                Arc::clone(&combined_middlewares),
            ));
        }

        for (nested_prefix, nested_router) in self.nested {
            let full_prefix = if prefix.is_empty() {
                nested_prefix.clone()
            } else {
                format!("{}{}", prefix, nested_prefix)
            };

            let nested_routes = nested_router.flatten_with_shared(
                base_prefix,
                &full_prefix,
                Some(&combined_middlewares),
            );
            flattened.extend(nested_routes);
        }

        flattened
    }
}

impl<S> Default for Router<S>
where
    S: Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
