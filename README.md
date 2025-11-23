<div align="center">
  <h1>Foton</h1>
  
  [![Crates.io](https://img.shields.io/crates/v/foton.svg)](https://crates.io/crates/foton)
  [![Documentation](https://docs.rs/foton/badge.svg)](https://docs.rs/foton)
  [![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)
  
  <p><strong>Fast, minimal, Rust-native web framework</strong></p>
</div>

---

## Overview

Foton is an asynchronous web framework built on Tokio and Hyper. It provides type-safe extractors, composable middleware, and WebSocket support.

## Installation

```toml
[dependencies]
foton = "0.0.5"
tokio = { version = "1", features = ["full"] }
```

## Examples

See the [`examples/`](examples/) directory:

- [streaming-demo](examples/streaming-demo)
- [websocket-echo](examples/websocket-echo)
- [file-serving](examples/file-serving)

## Resources

- [Documentation](https://docs.rs/foton)
- [GitHub Repository](https://github.com/erickweyunga/foton)
- [Crates.io](https://crates.io/crates/foton)

## Requirements

- Rust 1.85.0+

## License

Dual-licensed under MIT or Apache-2.0.
