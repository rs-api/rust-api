# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.4] - 2024-11-17

### Added
- `graceful-shutdown` allow more control and wait for requests to complete befor shutting down

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

### Ecosystem
- rust-api-cors: CORS middleware
- rust-api-error-handlers: Error handling utilities
- rust-api-helpers: Middleware composition helpers
- rust-api-client: HTTP client with modern Hyper 1.0 APIs
