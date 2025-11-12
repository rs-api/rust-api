use rust_api::prelude::*;

async fn home(_req: Req) -> Res {
    Res::html(
        r#"
<!DOCTYPE html>
<html>
<body>
    <h2>Form Handling Example</h2>
    <form action="/submit" method="post">
        Name: <input type="text" name="name"><br>
        Email: <input type="email" name="email"><br>
        <input type="submit" value="Submit">
    </form>
    <p>Try also: <a href="/search?q=rust&page=1">GET with query params</a></p>
</body>
</html>
    "#,
    )
}

async fn submit(_req: Req) -> Res {
    // Note: Full form parsing will be available when extractors are complete
    Res::json(&serde_json::json!({
        "message": "Form submitted successfully",
        "note": "Form extractors coming soon"
    }))
}

async fn search(_req: Req) -> Res {
    // Note: Query parameter extraction will be available when Query<T> is complete
    Res::json(&serde_json::json!({
        "message": "Search results",
        "note": "Query extractors coming soon"
    }))
}

#[tokio::main]
async fn main() {
    let app = RustApi::new()
        .max_body_size(1024 * 1024) // 1MB for forms (default is 64KB)
        .get("/", home)
        .post("/submit", submit)
        .get("/search", search);

    println!("Listening on http://127.0.0.1:3003");
    println!("Max body size: 1MB");
    app.listen(([127, 0, 0, 1], 3003))
        .await
        .expect("Failed to start server");
}
