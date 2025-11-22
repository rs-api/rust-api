# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2024-11-22

### Added
- **WebSocket Support**: Full RFC 6455 compliant WebSocket implementation
  - `WebSocketUpgrade` extractor for handling upgrade requests
  - `WebSocket` struct for bidirectional communication
  - Support for Text, Binary, Ping, Pong, and Close messages
  - Automatic frame encoding/decoding with masking
  - Available via `websocket` feature flag
  
- **File Serving**: `Res::file()` for streaming files from disk
  - Efficient streaming using `ReaderStream`
  - Automatic 404 handling for missing files
  - Manual MIME type control via `.header()`
  - Zero dependencies for MIME detection

## [0.0.5] - 2024-11-21

### 🚨 BREAKING CHANGES

Complete framework redesign with mutation-based API for maximum flexibility.

### Changed

- **API Redesign**: All builder methods converted to mutation methods
  - `RustApi::new()` now returns mutable instance
  - `.get()`, `.post()`, `.put()`, `.delete()`, `.patch()` now mutate instead of returning `Self`
  - `.layer()` now mutates instead of returning `Self`
  - `.nest()` now mutates instead of returning `Self`
  - `.route()` now mutates instead of returning `Self`
  
- **Router Redesign**: Router methods now mutate
  - `Router::get()`, `.post()`, etc. now take `&mut self`
  - `Router::layer()` now takes `&mut self`
  - `Router::nest()` now takes `&mut self`
  
- **Route Redesign**: Route middleware now uses mutation
  - `Route::layer()` now takes `&mut self`
  
- **Error Handler API**: Changed from builder to mutation
  - `RustApi::error_handler()` changed to `.set_error_handler(&mut self)`

### Added

- **Utility Methods**:
  - `RustApi::route_count()` - Get number of registered routes
  - `RustApi::has_route()` - Check if route exists at path
  - `Router::route_count()` - Get number of routes in router
  
- **Better Server Feedback**:
  - Console output when server starts: "🚀 Server listening on http://..."
  - Console output on graceful shutdown: "🛑 Shutting down gracefully..."
  - Console output on successful shutdown: "✅ Server shut down successfully"

### Improved

- **Documentation**: All public APIs now have comprehensive documentation with examples
- **Examples**: Updated README with mutation-based examples
- **Flexibility**: Mutation API enables:
  - Conditional route registration
  - Dynamic routing from configuration
  - Plugin systems
  - Multi-module applications
  - Loop-based route generation

### Migration Guide

#### Before (v0.0.4 - Builder API):

```rust
let app = RustApi::new()
    .layer(cors_middleware)
    .get("/", handler)
    .post("/users", create_user)
    .nest("/api", router)
    .listen(([127, 0, 0, 1], 3000))
    .await?;
```

#### After (v0.1.0 - Mutation API):

```rust
let mut app = RustApi::new();
app.layer(cors_middleware);
app.get("/", handler);
app.post("/users", create_user);
app.nest("/api", router);
app.listen(([127, 0, 0, 1], 3000)).await?;
```

#### Router Changes:

```rust
// Before
let router = Router::new()
    .get("/users", handler)
    .layer(middleware);

// After
let mut router = Router::new();
router.get("/users", handler);
router.layer(middleware);
```

#### Route Changes:

```rust
// Before
let route = Route::get("/admin", handler)
    .layer(auth_middleware);

// After
let mut route = Route::get("/admin", handler);
route.layer(auth_middleware);
```

---

## [0.0.4] - 2024-11-17

### Added
- `graceful-shutdown` - Allow more control and wait for requests to complete before shutting down

## [0.0.3] - 2024-11-13

### Added
- `from_fn()` helper for creating middleware from closures
- `When` conditional middleware helper (simplified from `ConditionalMiddleware`)

### Changed
- Simplified `.layer()` API to accept `Middleware` trait directly
- Improved middleware ergonomics with cleaner patterns

### Fixed
- Middleware composition after API encapsulation changes
- Type inference issues in closure-based middleware

## [0.0.1] - 2024-11-13

### Added
- Initial release
- Async HTTP server with Tokio runtime
- High-performance routing with radix tree
- Type-safe request extractors (JSON, Form, Query, Path)
- Flexible middleware system with per-route support
- Shared application state
- Streaming request bodies for zero-copy optimization
- Error handling with custom error handlers
- Response builders and helpers
- Extension system for request-scoped data
