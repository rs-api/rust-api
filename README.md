# Rust API

Fast and scalable web framework for Rust.

## Features

- Async by default with Tokio runtime
- High-performance routing with radix tree
- Type-safe request extractors
- Flexible middleware system with per-route support
- Built-in JSON, form, and query parameter handling
- Shared application state
- Streaming request bodies for zero-copy optimization

## Installation

Add to your Cargo.toml:

```toml
[dependencies]
rust-api = "0.0.3"
```

## Philosophy

Lightweight core with a rich ecosystem. The framework provides essential features while the community builds specialized middleware and extensions.

## Ecosystem

Official middleware packages:

- **rust-api-cors** - CORS middleware
- **rust-api-error-handlers** - Error handling utilities
- **rust-api-helpers** - Middleware composition helpers
- **rust-api-client** - HTTP client with modern Hyper APIs

## Documentation

Visit the examples directory for working demonstrations of all features.

## License

MIT
