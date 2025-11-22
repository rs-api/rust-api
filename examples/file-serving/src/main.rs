use rust_api::{Req, Res, RustApi};

async fn serve_html(_req: Req) -> Res {
    Res::file("examples/file-serving/static/index.html")
        .await
        .header("content-type", "text/html")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = RustApi::new();

    app.get("/", serve_html);

    println!("Server running on http://127.0.0.1:3000");
    app.listen(([127, 0, 0, 1], 3000)).await?;
    Ok(())
}
