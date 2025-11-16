//! HTTP application builder.

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
use tokio::signal;
use tokio::sync::watch;

use crate::{
    Error, ErrorHandler, Handler, IntoRes, Middleware, Req, Res, Result, Router,
    handler::IntoHandler,
};

type BoxedHandler<S> = Arc<dyn Handler<S>>;
type BoxedMiddleware<S> = Arc<dyn Middleware<S>>;
type SharedMiddlewares<S> = Arc<Vec<BoxedMiddleware<S>>>;
type BoxedErrorHandler = Arc<dyn ErrorHandler>;
type MethodHandlers<S> = HashMap<Method, (BoxedHandler<S>, SharedMiddlewares<S>)>;

/// HTTP application.
pub struct RustApi<S = ()> {
    routes: Vec<(Method, String, BoxedHandler<S>, SharedMiddlewares<S>)>,
    middlewares: Vec<BoxedMiddleware<S>>,
    state: Option<Arc<S>>,
    router: Option<matchit::Router<Arc<MethodHandlers<S>>>>,
    error_handler: Option<BoxedErrorHandler>,
}

impl RustApi<()> {
    /// Create application.
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
    /// Create application with state.
    pub fn with_state(state: S) -> Self {
        Self {
            routes: Vec::new(),
            middlewares: Vec::new(),
            state: Some(Arc::new(state)),
            router: None,
            error_handler: None,
        }
    }

    /// Set custom error handler.
    pub fn error_handler<H: ErrorHandler>(mut self, handler: H) -> Self {
        self.error_handler = Some(Arc::new(handler));
        self
    }

    /// Add global middleware.
    pub fn layer<M: crate::Middleware<S>>(mut self, middleware: M) -> Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    /// Register GET route.
    pub fn get<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes.push((
            Method::GET,
            path.to_string(),
            handler.into_handler(),
            Arc::new(Vec::new()),
        ));
        self
    }

    /// Register POST route.
    pub fn post<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes.push((
            Method::POST,
            path.to_string(),
            handler.into_handler(),
            Arc::new(Vec::new()),
        ));
        self
    }

    /// Register PUT route.
    pub fn put<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes.push((
            Method::PUT,
            path.to_string(),
            handler.into_handler(),
            Arc::new(Vec::new()),
        ));
        self
    }

    /// Register DELETE route.
    pub fn delete<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes.push((
            Method::DELETE,
            path.to_string(),
            handler.into_handler(),
            Arc::new(Vec::new()),
        ));
        self
    }

    /// Register PATCH route.
    pub fn patch<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: IntoHandler<S, T>,
    {
        self.routes.push((
            Method::PATCH,
            path.to_string(),
            handler.into_handler(),
            Arc::new(Vec::new()),
        ));
        self
    }

    /// Register route with per-route middleware.
    pub fn route(mut self, route: crate::Route<S>) -> Self {
        self.routes
            .push((route.method, route.path, route.handler, route.middlewares));
        self
    }

    /// Mount router at prefix.
    pub fn nest(mut self, prefix: &str, router: Router<S>) -> Self {
        let flattened = router.flatten(prefix);
        for (method, path, handler, middlewares) in flattened {
            self.routes.push((method, path, handler, middlewares));
        }
        self
    }

    fn build_router(mut self) -> Self {
        let mut router = matchit::Router::new();
        let mut path_methods: HashMap<String, MethodHandlers<S>> = HashMap::new();

        let global_middlewares = Arc::new(self.middlewares.clone());

        for (method, path, handler, route_middlewares) in self.routes.drain(..) {
            let combined_middlewares: SharedMiddlewares<S> = if route_middlewares.is_empty() {
                Arc::clone(&global_middlewares)
            } else if global_middlewares.is_empty() {
                route_middlewares
            } else {
                let mut combined =
                    Vec::with_capacity(global_middlewares.len() + route_middlewares.len());
                combined.extend_from_slice(&global_middlewares);
                combined.extend_from_slice(&route_middlewares);
                Arc::new(combined)
            };

            path_methods
                .entry(path.clone())
                .or_insert_with(HashMap::new)
                .insert(method, (handler, combined_middlewares));
        }

        for (path, methods) in path_methods {
            router.insert(&path, Arc::new(methods)).ok();
        }

        self.router = Some(router);
        self
    }

    /// Starts the HTTP server and listens for incoming connections.
    ///
    /// The server implements graceful shutdown by:
    /// - Listening for SIGTERM/SIGINT signals on Unix systems
    /// - Listening for Ctrl+C on all platforms
    /// - Stopping acceptance of new connections when a signal is received
    /// - Allowing in-flight requests to complete before terminating
    ///
    /// Each connection is handled in its own task and monitored via a watch channel.
    /// When shutdown is triggered, all active connections receive the signal and
    /// complete their current requests before the server exits.
    ///
    /// # Arguments
    ///
    /// * `addr` - Socket address to bind to (e.g., `([127, 0, 0, 1], 3000)`)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rust_api::RustApi;
    /// # #[tokio::main]
    /// # async fn main() -> rust_api::Result<()> {
    /// let app = RustApi::new()
    ///     .get("/", |_| async { Ok(rust_api::Res::text("Hello")) });
    ///
    /// app.listen(([127, 0, 0, 1], 3000)).await
    /// # }
    /// ```
    pub async fn listen(self, addr: impl Into<SocketAddr>) -> Result<()> {
        let addr = addr.into();
        let app = Arc::new(self.build_router());
        let listener = TcpListener::bind(addr).await?;

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

        tokio::spawn(async move {
            if let Err(e) = shutdown_signal().await {
                eprintln!("Error waiting for shutdown signal: {:?}", e);
            }
            let _ = shutdown_tx.send(true);
        });

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            let io = TokioIo::new(stream);
                            let app = Arc::clone(&app);
                            let mut shutdown_rx = shutdown_rx.clone();

                            tokio::task::spawn(async move {
                                let conn = http1::Builder::new()
                                    .serve_connection(
                                        io,
                                        service_fn(move |req| {
                                            let app = Arc::clone(&app);
                                            async move { app.handle_request(req).await }
                                        }),
                                    );

                                let mut conn = std::pin::pin!(conn);

                                tokio::select! {
                                    result = conn.as_mut() => {
                                        if let Err(err) = result {
                                            eprintln!("Error serving connection: {:?}", err);
                                        }
                                    }
                                    _ = shutdown_rx.changed() => {
                                        conn.as_mut().graceful_shutdown();
                                        let _ = conn.await;
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Error accepting connection: {:?}", e);
                        }
                    }
                }
                _ = shutdown_rx.changed() => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(
        &self,
        req: Request<Incoming>,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        let path = req.uri().path().to_string();
        let method = req.method().clone();
        let mut rust_req = Req::from_hyper(req);

        let response = match &self.router {
            Some(router) => match router.at(&path) {
                Ok(matched) => {
                    let mut params = HashMap::new();
                    for (key, value) in matched.params.iter() {
                        params.insert(key.to_string(), value.to_string());
                    }
                    rust_req.set_path_params(params);

                    if let Some(ref error_handler) = self.error_handler {
                        rust_req.extensions_mut().insert(Arc::clone(error_handler));
                    }

                    let method_handlers = matched.value;

                    match method_handlers.get(&method) {
                        Some((handler, middlewares)) => {
                            let state = match &self.state {
                                Some(s) => Arc::clone(s),
                                None => {
                                    return Ok(Error::internal("State not initialized")
                                        .into_res()
                                        .into_hyper());
                                }
                            };

                            if middlewares.is_empty() {
                                handler.call(rust_req, state).await
                            } else {
                                let handler_clone = Arc::clone(handler);
                                let mut next_fn: Arc<
                                    dyn Fn(
                                            Req,
                                            Arc<S>,
                                        )
                                            -> std::pin::Pin<
                                            Box<dyn std::future::Future<Output = Res> + Send>,
                                        > + Send
                                        + Sync,
                                > = Arc::new(move |req, state| {
                                    let handler = Arc::clone(&handler_clone);
                                    Box::pin(async move { handler.call(req, state).await })
                                });

                                for middleware in middlewares.iter().rev() {
                                    let middleware_clone = Arc::clone(middleware);
                                    let inner = Arc::clone(&next_fn);
                                    let state_for_middleware = Arc::clone(&state);

                                    next_fn = Arc::new(move |req, _state| {
                                        let mw = Arc::clone(&middleware_clone);
                                        let inner_clone = Arc::clone(&inner);
                                        let state_clone = Arc::clone(&state_for_middleware);

                                        Box::pin(async move {
                                            let next = crate::Next::new(
                                                inner_clone,
                                                Arc::clone(&state_clone),
                                            );
                                            mw.handle(req, state_clone, next).await
                                        })
                                    });
                                }

                                next_fn(rust_req, state).await
                            }
                        }
                        None => {
                            use crate::IntoRes;
                            let allowed_methods: Vec<String> = method_handlers
                                .keys()
                                .map(|m| m.as_str().to_string())
                                .collect();

                            let mut response = Error::method_not_allowed(&format!(
                                "Method {} not allowed. Allowed methods: {}",
                                method,
                                allowed_methods.join(", ")
                            ))
                            .into_res();

                            response
                                .headers_mut()
                                .insert("Allow", allowed_methods.join(", ").parse().unwrap());

                            response
                        }
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

/// Waits for system shutdown signals.
///
/// On Unix: Listens for SIGTERM and SIGINT signals.
/// On Windows: Listens for Ctrl+C via tokio::signal::ctrl_c.
///
/// Returns when any signal is received.
async fn shutdown_signal() -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;

        tokio::select! {
            _ = sigterm.recv() => {}
            _ = sigint.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        signal::ctrl_c().await?;
    }

    Ok(())
}
