//! Main application entry point
//!
//! The [`RustApi`] type is the core of the framework, providing
//! routing, middleware, and state management.

use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::{
    Error, ErrorHandler, Handler, IntoRes, Middleware, Req, Res, Result, Router,
    handler::IntoHandler, middleware::FnMiddleware,
};

type BoxedHandler<S> = Arc<dyn Handler<S>>;
type BoxedMiddleware<S> = Arc<dyn Middleware<S>>;
type BoxedErrorHandler = Arc<dyn ErrorHandler>;

/// Main application
pub struct RustApi<S = ()> {
    routes: Vec<(Method, String, BoxedHandler<S>)>,
    middlewares: Vec<BoxedMiddleware<S>>,
    state: Option<Arc<S>>,
    router: Option<matchit::Router<(BoxedHandler<S>, Vec<BoxedMiddleware<S>>)>>,
    error_handler: Option<BoxedErrorHandler>,
}

impl RustApi<()> {
    /// Create new application without state
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            middlewares: Vec::new(),
            state: Some(Arc::new(())),
            router: None,
            error_handler: None,
        }
    }
}

impl<S: Send + Sync + 'static> RustApi<S> {
    /// Create new application with state
    pub fn with_state(state: S) -> Self {
        Self {
            routes: Vec::new(),
            middlewares: Vec::new(),
            state: Some(Arc::new(state)),
            router: None,
            error_handler: None,
        }
    }

    /// Set a custom error handler
    ///
    /// The error handler controls how errors are converted into HTTP responses.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use rust_api::prelude::*;
    ///
    /// let app = RustApi::new()
    ///     .error_handler(JsonErrorHandler)
    ///     .get("/", handler);
    /// ```
    pub fn error_handler<H: ErrorHandler>(mut self, handler: H) -> Self {
        self.error_handler = Some(Arc::new(handler));
        self
    }

    /// Add global middleware
    pub fn layer<F, Fut>(mut self, middleware: F) -> Self
    where
        F: Fn(Req, Arc<S>, crate::Next<S>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = crate::Res> + Send + 'static,
    {
        self.middlewares.push(Arc::new(FnMiddleware(middleware)));
        self
    }

    /// Add GET route
    pub fn get<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::GET, path.to_string(), handler.into_handler()));
        self
    }

    /// Add POST route
    pub fn post<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::POST, path.to_string(), handler.into_handler()));
        self
    }

    /// Add PUT route
    pub fn put<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::PUT, path.to_string(), handler.into_handler()));
        self
    }

    /// Add DELETE route
    pub fn delete<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::DELETE, path.to_string(), handler.into_handler()));
        self
    }

    /// Add PATCH route
    pub fn patch<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes
            .push((Method::PATCH, path.to_string(), handler.into_handler()));
        self
    }

    /// Nest a router at a path prefix
    pub fn nest(mut self, prefix: &str, router: Router<S>) -> Self {
        let flattened = router.flatten(prefix);
        for (method, path, handler, mut middlewares) in flattened {
            let mut combined = self.middlewares.clone();
            combined.append(&mut middlewares);
            self.routes.push((method, path, handler));
        }
        self
    }

    /// Build the router
    fn build_router(mut self) -> Self {
        let mut router = matchit::Router::new();

        let mut method_routes: HashMap<
            Method,
            Vec<(String, BoxedHandler<S>, Vec<BoxedMiddleware<S>>)>,
        > = HashMap::new();

        for (method, path, handler) in self.routes.drain(..) {
            method_routes.entry(method).or_insert_with(Vec::new).push((
                path,
                handler,
                self.middlewares.clone(),
            ));
        }

        for (_method, routes) in method_routes {
            for (path, handler, middlewares) in routes {
                router.insert(&path, (handler, middlewares)).ok();
            }
        }

        self.router = Some(router);
        self
    }

    /// Start listening on address
    pub async fn listen(self, addr: impl Into<SocketAddr>) -> Result<()> {
        let addr = addr.into();
        let app = Arc::new(self.build_router());

        let listener = TcpListener::bind(addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let app = Arc::clone(&app);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(move |req| {
                            let app = Arc::clone(&app);
                            async move { app.handle_request(req).await }
                        }),
                    )
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }

    async fn handle_request(
        &self,
        req: Request<Incoming>,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        let path = req.uri().path().to_string();

        let mut rust_req = Req::from_hyper(req);

        rust_req = match rust_req.consume_body().await {
            Ok(r) => r,
            Err(e) => {
                use crate::IntoRes;
                return Ok(e.into_res().into_hyper());
            }
        };

        let response = match &self.router {
            Some(router) => match router.at(&path) {
                Ok(matched) => {
                    let mut params = HashMap::new();
                    for (key, value) in matched.params.iter() {
                        params.insert(key.to_string(), value.to_string());
                    }
                    rust_req.set_path_params(params);

                    // Store error handler in extensions if available
                    if let Some(ref error_handler) = self.error_handler {
                        rust_req.extensions_mut().insert(Arc::clone(error_handler));
                    }

                    let (handler, middlewares) = matched.value;
                    let state = match &self.state {
                        Some(s) => Arc::clone(s),
                        None => {
                            return Ok(Error::internal("State not initialized")
                                .into_res()
                                .into_hyper());
                        }
                    };

                    // Apply middleware chain
                    if middlewares.is_empty() {
                        handler.call(rust_req, state).await
                    } else {
                        // Build middleware chain from innermost (handler) to outermost
                        let handler_clone = Arc::clone(handler);
                        // let state_clone = Arc::clone(&state);

                        // Start with the handler as the innermost function
                        let mut next_fn: Arc<
                            dyn Fn(
                                    Req,
                                    Arc<S>,
                                ) -> std::pin::Pin<
                                    Box<dyn std::future::Future<Output = Res> + Send>,
                                > + Send
                                + Sync,
                        > = Arc::new(move |req, state| {
                            let handler = Arc::clone(&handler_clone);
                            Box::pin(async move { handler.call(req, state).await })
                        });

                        // Wrap each middleware around the chain (in reverse order)
                        for middleware in middlewares.iter().rev() {
                            let middleware_clone = Arc::clone(middleware);
                            let inner = Arc::clone(&next_fn);
                            let state_for_middleware = Arc::clone(&state);

                            next_fn = Arc::new(move |req, _state| {
                                let mw = Arc::clone(&middleware_clone);
                                let inner_clone = Arc::clone(&inner);
                                let state_clone = Arc::clone(&state_for_middleware);

                                Box::pin(async move {
                                    let next =
                                        crate::Next::new(inner_clone, Arc::clone(&state_clone));
                                    mw.handle(req, state_clone, next).await
                                })
                            });
                        }

                        // Execute the middleware chain
                        next_fn(rust_req, state).await
                    }
                }
                Err(_) => {
                    use crate::IntoRes;
                    Error::not_found("Route not found").into_res()
                }
            },
            None => {
                use crate::IntoRes;
                Error::internal("Router not initialized").into_res()
            }
        };

        Ok(response.into_hyper())
    }
}

impl<S> Default for RustApi<S>
where
    S: Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            routes: Vec::new(),
            middlewares: Vec::new(),
            state: None,
            router: None,
            error_handler: None,
        }
    }
}
