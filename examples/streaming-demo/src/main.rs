use foton::{Foton, Req, Res, StreamSender};
use tokio::time::{Duration, sleep};

async fn index(_req: Req) -> Res {
    Res::html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Streaming Demo</title>
        </head>
        <body>
            <h1>Response Streaming Examples</h1>
            <ul>
                <li><a href="/stream">Basic Streaming</a> - Stream chunks over time</li>
                <li><a href="/countdown">Countdown</a> - Live countdown stream</li>
                <li><a href="/sse">Server-Sent Events</a> - SSE example</li>
            </ul>
        </body>
        </html>
        "#,
    )
}

async fn stream_handler(_req: Req) -> Res {
    Res::stream(|mut tx: StreamSender| async move {
        for i in 1..=10 {
            let chunk = format!("Chunk {}\n", i);
            tx.send_text(chunk).await.ok();
            sleep(Duration::from_millis(500)).await;
        }
        tx.send_text("Stream complete!\n").await.ok();
    })
}

async fn countdown_handler(_req: Req) -> Res {
    Res::stream(|mut tx: StreamSender| async move {
        tx.send_text("<!DOCTYPE html><html><body><h1>Countdown</h1><pre>")
            .await
            .ok();

        for i in (1..=10).rev() {
            tx.send_text(format!("{}\n", i)).await.ok();
            sleep(Duration::from_secs(1)).await;
        }

        tx.send_text("Blast off! 🚀</pre></body></html>").await.ok();
    })
}

async fn sse_handler(_req: Req) -> Res {
    let mut res = Res::stream(|mut tx: StreamSender| async move {
        for i in 1..=20 {
            let event = format!(
                "data: {{\"count\": {}, \"timestamp\": {}}}\n\n",
                i,
                i * 1000
            );
            tx.send_text(event).await.ok();
            sleep(Duration::from_millis(1000)).await;
        }
    });

    res.headers_mut()
        .insert("content-type", "text/event-stream".parse().unwrap());
    res.headers_mut()
        .insert("cache-control", "no-cache".parse().unwrap());
    res.headers_mut()
        .insert("connection", "keep-alive".parse().unwrap());

    res
}

#[tokio::main]
async fn main() {
    let mut app = Foton::new();

    app.get("/", index);
    app.get("/stream", stream_handler);
    app.get("/countdown", countdown_handler);
    app.get("/sse", sse_handler);

    println!("Streaming demo server listening on http://127.0.0.1:3000");
    println!("Visit http://127.0.0.1:3000 to see examples");

    app.listen(([127, 0, 0, 1], 3000)).await.unwrap();
}
