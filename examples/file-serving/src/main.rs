use foton::{Foton, Req, Res};

async fn serve_html(_req: Req) -> Res {
    Res::file("examples/file-serving/static/index.html").await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Foton::new();

    app.get("/", serve_html);

    println!("Server running on http://127.0.0.1:3000");
    app.listen(([127, 0, 0, 1], 3000)).await?;
    Ok(())
}
