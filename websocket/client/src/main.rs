use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use url::Url;



#[tokio::main]
async fn main() {
    let url = Url::parse("ws://127.0.0.1:8080").expect("Invalid URL");

    // connect_async は IntoClientRequest を実装している型 (&str など) を期待しているため、as_str() を使って &str に変換する必要がある
    let (mut ws_stream, _) = connect_async(url.as_str())
        .await
        .expect("Failed to connect");

    // サーバーにメッセージを送信
    ws_stream.send(Message::Text("Hello from client!".to_string().into()))
        .await
        .expect("Failed to send message");

    // サーバーからのレスポンスを受け取る
    if let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received from server: {}", text);
            }
            _ => {}
        }
    }
}
