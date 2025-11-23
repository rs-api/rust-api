//! HTTP application.

use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::res::BoxBody;
use hyper::body::Incoming;
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper::{Method, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::watch;

use crate::{
    Error, ErrorHandler, Handler, IntoRes, Middleware, Req, Res, Result, Router, ServerConfig,
    handler::IntoHandler,
};

type BoxedHandler<S> = Arc<dyn Handler<S>>;
type BoxedMiddleware<S> = Arc<dyn Middleware<S>>;
type SharedMiddlewares<S> = Arc<Vec<BoxedMiddleware<S>>>;
type BoxedErrorHandler = Arc<dyn ErrorHandler>;
type MethodHandlers<S> = HashMap<Method, (BoxedHandler<S>, SharedMiddlewares<S>)>;

/// HTTP application.
pub struct Foton<S = ()> {
    routes: Vec<(Method, String, BoxedHandler<S>, SharedMiddlewares<S>)>,
    middlewares: Vec<BoxedMiddleware<S>>,
    state: Option<Arc<S>>,
    router: Option<matchit::Router<Arc<MethodHandlers<S>>>>,
    error_handler: Option<BoxedErrorHandler>,

    // Configuration
    body_limit: Option<usize>,
    request_timeout: Option<Duration>,
    handler_timeout: Option<Duration>,
    http2_enabled: bool,
    max_connections: Option<usize>,
    keep_alive: Option<Duration>,
}

impl Foton<()> {
    /// Create a new application with default state.
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            middlewares: Vec::new(),
            state: Some(Arc::new(())),
            router: None,
            error_handler: None,
            body_limit: None,
            request_timeout: None,
            handler_timeout: None,
            http2_enabled: false,
            max_connections: None,
            keep_alive: None,
        }
    }
}

impl<S: Send + Sync + 'static> Foton<S> {
    /// Create application with custom state.
    ///
    /// State is shared across handlers via `Arc<S>` and accessed using `State<S>` extractor.
    pub fn with_state(state: S) -> Self {
        Self {
            routes: Vec::new(),
            middlewares: Vec::new(),
            state: Some(Arc::new(state)),
            router: None,
            error_handler: None,
            body_limit: None,
            request_timeout: None,
            handler_timeout: None,
            http2_enabled: false,
            max_connections: None,
            keep_alive: None,
        }
    }

    /// Set custom error handler.
    pub fn set_error_handler<H: ErrorHandler>(&mut self, handler: H) {
        self.error_handler = Some(Arc::new(handler));
    }

    /// Attach global middleware.
    ///
    /// Middleware runs for all routes. Execution order matches registration order.
    pub fn attach<M: Middleware<S>>(&mut self, middleware: M) {
        self.middlewares.push(Arc::new(middleware));
    }

    /// Register a GET route.
    pub fn get<H, T>(&mut self, path: &str, handler: H)
    where
        H: IntoHandler<S, T>,
    {
        self.routes.push((
            Method::GET,
            path.to_string(),
            handler.into_handler(),
            Arc::new(Vec::new()),
        ));
    }

    /// Register a POST route.
    pub fn post<H, T>(&mut self, path: &str, handler: H)
    where
        H: IntoHandler<S, T>,
    {
        self.routes.push((
            Method::POST,
            path.to_string(),
            handler.into_handler(),
            Arc::new(Vec::new()),
        ));
    }

    /// Register a PUT route.
    pub fn put<H, T>(&mut self, path: &str, handler: H)
    where
        H: IntoHandler<S, T>,
    {
        self.routes.push((
            Method::PUT,
            path.to_string(),
            handler.into_handler(),
            Arc::new(Vec::new()),
        ));
    }

    /// Register a DELETE route.
    pub fn delete<H, T>(&mut self, path: &str, handler: H)
    where
        H: IntoHandler<S, T>,
    {
        self.routes.push((
            Method::DELETE,
            path.to_string(),
            handler.into_handler(),
            Arc::new(Vec::new()),
        ));
    }

    /// Register a PATCH route.
    pub fn patch<H, T>(&mut self, path: &str, handler: H)
    where
        H: IntoHandler<S, T>,
    {
        self.routes.push((
            Method::PATCH,
            path.to_string(),
            handler.into_handler(),
            Arc::new(Vec::new()),
        ));
    }

    /// Register a route with per-route middleware.
    pub fn route(&mut self, route: crate::Route<S>) {
        self.routes
            .push((route.method, route.path, route.handler, route.middlewares));
    }

    /// Mount a router at a prefix.
    pub fn nest(&mut self, prefix: &str, router: Router<S>) {
        let flattened = router.flatten(prefix);
        for (method, path, handler, middlewares) in flattened {
            self.routes.push((method, path, handler, middlewares));
        }
    }

    /// Get the number of registered routes.
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Check if a route exists at the given path.
    pub fn has_route(&self, path: &str) -> bool {
        self.routes.iter().any(|(_, p, _, _)| p == path)
    }

    /// Set maximum request body size in bytes.
    pub fn set_body_limit(&mut self, limit: usize) {
        self.body_limit = Some(limit);
    }

    /// Set request timeout duration.
    pub fn set_request_timeout(&mut self, timeout: Duration) {
        self.request_timeout = Some(timeout);
    }

    /// Set handler execution timeout duration.
    pub fn set_handler_timeout(&mut self, timeout: Duration) {
        self.handler_timeout = Some(timeout);
    }

    /// Enable or disable HTTP/2 support.
    pub fn set_http2(&mut self, enabled: bool) {
        self.http2_enabled = enabled;
    }

    /// Set maximum number of concurrent connections.
    pub fn set_max_connections(&mut self, max: usize) {
        self.max_connections = Some(max);
    }

    /// Set TCP keep-alive duration.
    pub fn set_keep_alive(&mut self, duration: Duration) {
        self.keep_alive = Some(duration);
    }

    /// Apply configuration from a config struct.
    pub fn apply_config(&mut self, config: ServerConfig) {
        if let Some(limit) = config.body_limit {
            self.body_limit = Some(limit);
        }
        if let Some(timeout) = config.request_timeout {
            self.request_timeout = Some(timeout);
        }
        if let Some(timeout) = config.handler_timeout {
            self.handler_timeout = Some(timeout);
        }
        self.http2_enabled = config.http2;
        if let Some(max) = config.max_connections {
            self.max_connections = Some(max);
        }
        self.keep_alive = config.keep_alive;
    }

    fn build_router(&mut self) {
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
    }

    /// Start the HTTP server.
    ///
    /// Implements graceful shutdown on SIGTERM/SIGINT signals.
    /// In-flight requests complete before the server terminates.
    pub async fn listen(mut self, addr: impl Into<SocketAddr>) -> Result<()> {
        let addr = addr.into();
        self.build_router();
        let app = Arc::new(self);
        let listener = TcpListener::bind(addr).await?;

        let active_connections = Arc::new(AtomicUsize::new(0));

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

        tokio::spawn(async move {
            let _ = shutdown_signal().await;
            let _ = shutdown_tx.send(true);
        });

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            // Check max connections limit
                            if let Some(max) = app.max_connections {
                                let current = active_connections.load(Ordering::Relaxed);
                                if current >= max {
                                    drop(stream);
                                    continue;
                                }
                            }

                            // Increment active connections
                            active_connections.fetch_add(1, Ordering::Relaxed);

                            let io = TokioIo::new(stream);
                            let app = Arc::clone(&app);
                            let mut shutdown_rx = shutdown_rx.clone();
                            let active_connections = Arc::clone(&active_connections);
                            let http2_enabled = app.http2_enabled;

                            tokio::task::spawn(async move {
                                if http2_enabled {
                                    let conn = http2::Builder::new(hyper_util::rt::TokioExecutor::new())
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
                                            let _ = result;
                                        }
                                        _ = shutdown_rx.changed() => {
                                            conn.as_mut().graceful_shutdown();
                                            let _ = conn.await;
                                        }
                                    }
                                } else {
                                    let conn = http1::Builder::new()
                                        .serve_connection(
                                            io,
                                            service_fn(move |req| {
                                                let app = Arc::clone(&app);
                                                async move { app.handle_request(req).await }
                                            }),
                                        )
                                        .with_upgrades();

                                    let mut conn = std::pin::pin!(conn);

                                    tokio::select! {
                                        result = conn.as_mut() => {
                                            let _ = result;
                                        }
                                        _ = shutdown_rx.changed() => {
                                            conn.as_mut().graceful_shutdown();
                                            let _ = conn.await;
                                        }
                                    }
                                }

                                // Decrement active connections when done
                                active_connections.fetch_sub(1, Ordering::Relaxed);
                            });
                        }
                        Err(_) => {}
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
    ) -> std::result::Result<Response<BoxBody>, Infallible> {
        let path = req.uri().path().to_string();
        let method = req.method().clone();
        let mut rust_req = Req::from_hyper(req);

        // Set body limit if configured
        rust_req.set_body_limit(self.body_limit);

        // Extract upgrade future before rust_req is moved
        #[cfg(feature = "websocket")]
        let on_upgrade = rust_req.take_upgrade();

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

                            // Execute handler with optional timeout
                            let handler_future = if middlewares.is_empty() {
                                Box::pin(handler.call(rust_req, state))
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

                                Box::pin(next_fn(rust_req, state))
                            };

                            // Apply handler timeout if configured
                            if let Some(timeout) = self.handler_timeout {
                                match tokio::time::timeout(timeout, handler_future).await {
                                    Ok(res) => res,
                                    Err(_) => {
                                        use crate::IntoRes;
                                        Error::Custom(format!(
                                            "Handler timeout after {:?}",
                                            timeout
                                        ))
                                        .into_res()
                                    }
                                }
                            } else {
                                handler_future.await
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

        // Check for WebSocket upgrade
        #[cfg(feature = "websocket")]
        {
            let mut response_mut = response;
            if let Some(ws_callback) = response_mut.take_ws_callback() {
                if let Some(upgrade_future) = on_upgrade {
                    tokio::task::spawn(async move {
                        match upgrade_future.await {
                            Ok(upgraded) => {
                                let ws = crate::websocket::WebSocket::new(upgraded);
                                ws_callback(ws).await;
                            }
                            Err(_e) => {
                                // Upgrade failed
                            }
                        }
                    });
                }
            }
            return Ok(response_mut.into_hyper());
        }

        #[cfg(not(feature = "websocket"))]
        Ok(response.into_hyper())
    }
}

impl<S> Default for Foton<S>
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
            body_limit: None,
            request_timeout: None,
            handler_timeout: None,
            http2_enabled: false,
            max_connections: None,
            keep_alive: None,
        }
    }
}

/// Create a new HTTP application.
///
/// # Example
///
/// ```rust
/// use foton::app;
///
/// let mut app = app();
/// app.get("/", |_| async { "Hello" });
/// ```
pub fn app() -> Foton {
    Foton::new()
}

/// Create an HTTP application with custom state.
///
/// State is shared across all handlers and accessed via the `State<S>` extractor.
///
/// # Example
///
/// ```rust
/// use foton::{app_with_state, State};
///
/// struct AppState {
///     db: Database,
/// }
///
/// let mut app = app_with_state(AppState { db });
/// app.get("/", |State(state): State<AppState>| async move {
///     // Access state.db
/// });
/// ```
pub fn app_with_state<S: Send + Sync + 'static>(state: S) -> Foton<S> {
    Foton::with_state(state)
}

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
