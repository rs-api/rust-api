# Rust API

Fast and scalable web framework for Rust with a mutation-based API for maximum flexibility.

## Features

- ✨ **Async by default** - Built on Tokio runtime
- 🚀 **High-performance routing** - Radix tree with O(log n) matching
- 🔒 **Type-safe extractors** - Compile-time guarantees for request data
- 🔌 **Flexible middleware** - Global, per-router, and per-route support
- 📦 **Built-in extractors** - JSON, Form, Query, Path, Headers, State
- 🌐 **Shared state** - Thread-safe state management with Arc
- 🛑 **Graceful shutdown** - Clean connection handling on SIGTERM/SIGINT
- 🔧 **Mutation-based API** - Maximum flexibility for dynamic routing

## Installation

Add to your Cargo.toml:

```toml
[dependencies]
rust-api = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use rust_api::{RustApi, Res};

#[tokio::main]
async fn main() -> rust_api::Result<()> {
    let mut app = RustApi::new();
    
    app.get("/", |_| async {
        Res::text("Hello, World!")
    });
    
    app.listen(([127, 0, 0, 1], 3000)).await
}
```

## Examples

### Basic Routing

```rust
use rust_api::{RustApi, Res, Req};

let mut app = RustApi::new();

// GET route
app.get("/users", |_req: Req| async {
    Res::json(&serde_json::json!({
        "users": ["Alice", "Bob"]
    }))
});

// POST route
app.post("/users", |_req: Req| async {
    Res::text("User created")
});

// Route with path parameters
app.get("/users/:id", |req: Req| async move {
    let id = req.param("id").unwrap_or("unknown");
    Res::text(&format!("User ID: {}", id))
});
```

### Type-Safe Extractors

```rust
use rust_api::{RustApi, Res, extractors::{Json, Query, Path}};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

let mut app = RustApi::new();

// JSON body extraction
app.post("/users", |Json(data): Json<CreateUser>| async move {
    let user = User {
        id: 1,
        name: data.name,
        email: data.email,
    };
    Res::json(&user)
});

// Query parameters
#[derive(Deserialize)]
struct Pagination {
    page: u32,
    limit: u32,
}

app.get("/users", |Query(params): Query<Pagination>| async move {
    Res::json(&serde_json::json!({
        "page": params.page,
        "limit": params.limit
    }))
});

// Path parameters
#[derive(Deserialize)]
struct UserPath {
    id: u64,
}

app.get("/users/:id", |Path(params): Path<UserPath>| async move {
    Res::text(&format!("User ID: {}", params.id))
});
```

### Application State

```rust
use rust_api::{RustApi, Res, extractors::State};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db_pool: String,
    api_key: String,
}

let state = AppState {
    db_pool: "postgres://localhost/mydb".to_string(),
    api_key: "secret123".to_string(),
};

let mut app = RustApi::with_state(state);

app.get("/config", |State(state): State<AppState>| async move {
    Res::json(&serde_json::json!({
        "db": state.db_pool
    }))
});
```

### Middleware

```rust
use rust_api::{RustApi, Res, middleware, Next, Req};

let mut app = RustApi::new();

// Global middleware
app.layer(middleware(|req: Req, state, next: Next<()>| async move {
    println!("Request: {} {}", req.method(), req.path());
    let res = next.run(req, state).await;
    println!("Response sent");
    res
}));

// CORS middleware
app.layer(middleware(|req: Req, state, next: Next<()>| async move {
    let mut res = next.run(req, state).await;
    res.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    res
}));
```

### Router Grouping

```rust
use rust_api::{RustApi, Router, Res};

let mut app = RustApi::new();

// Create API v1 router
let mut v1_router = Router::new();
v1_router.get("/users", |_| async {
    Res::text("V1 users")
});
v1_router.get("/posts", |_| async {
    Res::text("V1 posts")
});

// Create API v2 router
let mut v2_router = Router::new();
v2_router.get("/users", |_| async {
    Res::text("V2 users")
});

// Mount routers
app.nest("/api/v1", v1_router);
app.nest("/api/v2", v2_router);

// Routes available at:
// - /api/v1/users
// - /api/v1/posts
// - /api/v2/users
```

### Per-Route Middleware

```rust
use rust_api::{RustApi, Route, Res, middleware};

let mut app = RustApi::new();

// Create route with middleware
let mut admin_route = Route::get("/admin", |_| async {
    Res::text("Admin panel")
});

// Add authentication middleware
admin_route.layer(middleware(|req, state, next| async move {
    if req.header("Authorization").is_none() {
        return Res::status(401);
    }
    next.run(req, state).await
}));

app.route(admin_route);
```

### Dynamic Route Registration

```rust
use rust_api::{RustApi, Res};

let mut app = RustApi::new();

// Conditional routes
let enable_debug = std::env::var("DEBUG").is_ok();
if enable_debug {
    app.get("/debug", |_| async {
        Res::text("Debug info")
    });
}

// Dynamic routes from config
let routes = vec![
    ("/home", "Home page"),
    ("/about", "About page"),
    ("/contact", "Contact page"),
];

for (path, content) in routes {
    let content = content.to_string();
    app.get(path, move |_| {
        let content = content.clone();
        async move { Res::text(&content) }
    });
}
```

### Error Handling

```rust
use rust_api::{RustApi, Res, Error, extractors::Json};
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    name: String,
}

let mut app = RustApi::new();

app.post("/users", |Json(data): Json<CreateUser>| async move {
    if data.name.is_empty() {
        return Err(Error::bad_request("Name cannot be empty"));
    }
    
    if data.name.len() < 3 {
        return Err(Error::unprocessable("Name too short"));
    }
    
    Ok(Res::json(&serde_json::json!({
        "id": 1,
        "name": data.name
    })))
});
```

### Plugin System

```rust
use rust_api::{RustApi, Res, middleware};

// Define a plugin trait
trait Plugin {
    fn install(&self, app: &mut RustApi<()>);
}

// Authentication plugin
struct AuthPlugin;
impl Plugin for AuthPlugin {
    fn install(&self, app: &mut RustApi<()>) {
        app.layer(middleware(|req, state, next| async move {
            println!("Auth check");
            next.run(req, state).await
        }));
        
        app.post("/login", |_| async {
            Res::text("Login successful")
        });
    }
}

// CORS plugin
struct CorsPlugin;
impl Plugin for CorsPlugin {
    fn install(&self, app: &mut RustApi<()>) {
        app.layer(middleware(|req, state, next| async move {
            let mut res = next.run(req, state).await;
            res.headers_mut().insert(
                "Access-Control-Allow-Origin",
                "*".parse().unwrap()
            );
            res
        }));
    }
}

// Use plugins
let mut app = RustApi::new();
AuthPlugin.install(&mut app);
CorsPlugin.install(&mut app);
```

## Philosophy

**Lightweight core with maximum flexibility.**

The framework provides essential features with a mutation-based API that adapts to your needs:
- ✅ Simple apps with linear route definitions
- ✅ Complex apps with conditional logic
- ✅ Dynamic routing from configuration files
- ✅ Plugin systems with modular architectures
- ✅ Multi-module applications

## Performance

- **Zero-copy optimizations** - Static header values, body caching
- **Efficient routing** - O(log n) path matching with radix tree
- **Minimal allocations** - Pre-allocated capacities throughout
- **No unsafe code** - Memory safety without performance loss

## Documentation

For more examples and detailed documentation, visit [docs.rs/rust-api](https://docs.rs/rust-api).

## License

MIT
