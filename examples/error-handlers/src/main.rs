use rust_api::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Example 1: Using error handler with middleware to intercept errors
    let app_json = RustApi::new()
        .error_handler(JsonErrorHandler)
        // Middleware that uses the error handler from extensions
        .layer(|req, _state, next| async move {
            let res = next.run(req).await;
            res
        })
        .get("/json/bad-request", |_req: Req| async {
            Err::<Res, _>(Error::bad_request("Invalid JSON payload"))
        })
        .get("/json/unauthorized", |_req: Req| async {
            Err::<Res, _>(Error::unauthorized("Missing authentication token"))
        })
        .get("/json/not-found", |_req: Req| async {
            Err::<Res, _>(Error::not_found("Resource not found"))
        })
        .get("/json/internal", |_req: Req| async {
            Err::<Res, _>(Error::internal("Database connection failed"))
        });

    // Example 2: Custom HTML error pages
    struct HtmlErrorHandler;

    impl ErrorHandler for HtmlErrorHandler {
        fn handle(&self, error: Error) -> Res {
            let (status_code, message) = match &error {
                Error::Status(code, Some(msg)) => (*code, msg.clone()),
                Error::Status(code, None) => (*code, status_text(*code)),
                Error::Json(e) => (400, format!("JSON error: {}", e)),
                Error::Hyper(e) => (500, format!("HTTP error: {}", e)),
                Error::Io(e) => (500, format!("IO error: {}", e)),
                Error::Custom(msg) => (500, msg.clone()),
            };

            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>Error {}</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
        }}
        .error-container {{
            background: white;
            border-radius: 20px;
            padding: 40px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            max-width: 500px;
            text-align: center;
        }}
        .error-code {{
            font-size: 80px;
            font-weight: bold;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            margin: 0;
        }}
        .error-message {{
            font-size: 18px;
            color: #666;
            margin: 20px 0;
        }}
        .back-link {{
            display: inline-block;
            margin-top: 20px;
            padding: 12px 30px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            text-decoration: none;
            border-radius: 25px;
            transition: transform 0.2s;
        }}
        .back-link:hover {{
            transform: translateY(-2px);
        }}
    </style>
</head>
<body>
    <div class="error-container">
        <h1 class="error-code">{}</h1>
        <div class="error-message">{}</div>
        <a href="/" class="back-link">Go Home</a>
    </div>
</body>
</html>"#,
                status_code, status_code, message
            );

            Res::builder()
                .status(status_code)
                .header("Content-Type", "text/html; charset=utf-8")
                .text(html)
        }
    }

    let app_html = RustApi::new()
        .error_handler(HtmlErrorHandler)
        .get("/html/forbidden", |_req: Req| async {
            Err::<Res, _>(Error::forbidden("Access denied"))
        })
        .get("/html/not-found", |_req: Req| async {
            Err::<Res, _>(Error::not_found("Page not found"))
        });

    // Example 3: Using the stored error handler via middleware
    let app_with_middleware = RustApi::new()
        .error_handler(JsonErrorHandler)
        // Error handling middleware
        .layer(|req, _state, next| async move {
            let _error_handler = req.extensions().get::<Arc<dyn ErrorHandler>>().cloned();

            let res = next.run(req).await;

            // If response indicates an error and we have a custom handler,
            // we could transform it here. For now, the default behavior works.
            res
        })
        .get("/api/error", |_req: Req| async {
            Err::<Res, _>(Error::bad_request("Validation failed"))
        });

    tokio::select! {
        _ = app_json.listen(([127, 0, 0, 1], 3011)) => {},
        _ = app_html.listen(([127, 0, 0, 1], 3012)) => {},
        _ = app_with_middleware.listen(([127, 0, 0, 1], 3013)) => {},
    }
}

fn status_text(code: u16) -> String {
    match code {
        400 => "Bad Request".to_string(),
        401 => "Unauthorized".to_string(),
        403 => "Forbidden".to_string(),
        404 => "Not Found".to_string(),
        422 => "Unprocessable Entity".to_string(),
        500 => "Internal Server Error".to_string(),
        _ => format!("Error {}", code),
    }
}
