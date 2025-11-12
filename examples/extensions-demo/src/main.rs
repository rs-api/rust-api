use rust_api::prelude::*;

// Custom types to store in extensions
#[derive(Clone, Debug)]
struct User {
    id: u64,
    username: String,
    role: String,
}

#[derive(Clone, Debug)]
struct RequestId(String);

#[derive(Clone, Debug)]
struct AuthToken(String);

#[tokio::main]
async fn main() {
    let app = RustApi::new()
        // Middleware 1: Generate and attach request ID
        .layer(|mut req, _state, next| async move {
            let request_id = RequestId(format!("req-{}", uuid()));
            println!("[RequestId] Generated: {:?}", request_id);
            req.extensions_mut().insert(request_id);
            next.run(req).await
        })
        // Middleware 2: Extract auth token from header
        .layer(|mut req, _state, next| async move {
            if let Some(auth_header) = req.header("authorization") {
                let token = AuthToken(auth_header.to_string());
                println!("[Auth] Found token: {:?}", token);
                req.extensions_mut().insert(token);
            }
            next.run(req).await
        })
        // Middleware 3: Authenticate user based on token
        .layer(|mut req, _state, next| async move {
            // Check if we have an auth token
            if let Some(_token) = req.extensions().get::<AuthToken>() {
                // In a real app, you'd validate the token here
                // For demo, we'll just create a fake user
                let user = User {
                    id: 42,
                    username: "alice".to_string(),
                    role: "admin".to_string(),
                };
                println!("[Auth] Authenticated user: {:?}", user);
                req.extensions_mut().insert(user);
            } else {
                println!("[Auth] No token found, proceeding as anonymous");
            }
            next.run(req).await
        })
        // Route 1: Public route
        .get("/", |req: Req| async move {
            let request_id = req.extensions().get::<RequestId>();
            let user = req.extensions().get::<User>();

            let message = match (request_id, user) {
                (Some(req_id), Some(user)) => {
                    format!(
                        "Hello, {}! (Request: {:?}, Role: {})",
                        user.username, req_id.0, user.role
                    )
                }
                (Some(req_id), None) => {
                    format!("Hello, anonymous! (Request: {:?})", req_id.0)
                }
                _ => "Hello!".to_string(),
            };

            Res::text(message)
        })
        // Route 2: Protected route that requires user
        .get("/admin", |req: Req| async move {
            match req.extensions().get::<User>() {
                Some(user) if user.role == "admin" => {
                    Res::text(format!("Welcome to admin panel, {}!", user.username))
                }
                Some(user) => Res::builder()
                    .status(403)
                    .text(format!("Access denied. Role: {}", user.role)),
                None => Res::builder()
                    .status(401)
                    .text("Unauthorized: Please provide authentication"),
            }
        })
        // Route 3: Show all extension data
        .get("/debug", |req: Req| async move {
            let request_id = req.extensions().get::<RequestId>();
            let user = req.extensions().get::<User>();
            let token = req.extensions().get::<AuthToken>();

            let debug_info = format!(
                "Extension Debug Info:\n\
                - Request ID: {:?}\n\
                - User: {:?}\n\
                - Token: {:?}",
                request_id, user, token
            );

            Res::text(debug_info)
        });

    println!("Server starting on http://127.0.0.1:3007");
    println!("");
    println!("Try these requests:");
    println!("  1. curl http://127.0.0.1:3007/");
    println!("     -> Anonymous request");
    println!("");
    println!("  2. curl -H 'Authorization: Bearer token123' http://127.0.0.1:3007/");
    println!("     -> Authenticated request");
    println!("");
    println!("  3. curl http://127.0.0.1:3007/admin");
    println!("     -> Should be unauthorized");
    println!("");
    println!("  4. curl -H 'Authorization: Bearer token123' http://127.0.0.1:3007/admin");
    println!("     -> Should succeed (admin access)");
    println!("");
    println!("  5. curl http://127.0.0.1:3007/debug");
    println!("     -> Show extension data");
    println!("");

    app.listen(([127, 0, 0, 1], 3007)).await.unwrap();
}

// Simple UUID generator for demo purposes
fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    format!("{:016x}", timestamp)
}
