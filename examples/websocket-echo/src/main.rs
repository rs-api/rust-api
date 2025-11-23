use foton::{Foton, Message, Req, Res, WebSocket, WebSocketUpgrade};

async fn index(_req: Req) -> Res {
    Res::html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>WebSocket Echo</title>
        </head>
        <body>
            <h1>WebSocket Echo Server</h1>
            <input id="message" type="text" placeholder="Enter message">
            <button onclick="send()">Send</button>
            <div id="output"></div>

            <script>
                const ws = new WebSocket('ws://localhost:3000/ws');
                const output = document.getElementById('output');

                ws.onopen = () => {
                    output.innerHTML += '<p>Connected to server</p>';
                };

                ws.onmessage = (event) => {
                    output.innerHTML += `<p>Received: ${event.data}</p>`;
                };

                ws.onclose = () => {
                    output.innerHTML += '<p>Disconnected from server</p>';
                };

                function send() {
                    const msg = document.getElementById('message').value;
                    ws.send(msg);
                    output.innerHTML += `<p>Sent: ${msg}</p>`;
                }
            </script>
        </body>
        </html>
        "#,
    )
}

async fn handle_websocket(mut ws: WebSocket) {
    while let Ok(Some(msg)) = ws.receive().await {
        match msg {
            Message::Text(text) => {
                let _ = ws.send_text(format!("Echo: {}", text)).await;
            }
            Message::Binary(data) => {
                let _ = ws.send_binary(data).await;
            }
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }
}

async fn websocket_handler(ws: WebSocketUpgrade) -> Res {
    ws.upgrade(|socket| Box::pin(handle_websocket(socket)))
}

#[tokio::main]
async fn main() {
    let mut app = Foton::new();

    app.get("/", index);
    app.get("/ws", websocket_handler);

    println!("WebSocket echo server listening on http://127.0.0.1:3000");
    println!("Open http://127.0.0.1:3000 in your browser");

    app.listen(([127, 0, 0, 1], 3000)).await.unwrap();
}
