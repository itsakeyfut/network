use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::task;

async fn handle_connection(stream: TcpStream) {
    let ws_stream = accept_async(stream)
        .await
        .expect("Failed to accept");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received: {}", text);
                // レスポンスを送信
                ws_sender.send(Message::Text("Response".to_string().into()))
                    .await
                    .unwrap();
            }
            Ok(Message::Close(_)) => {
                // クローズメッセージが来た場合
                println!("Connection closed by client");
                break;
            }
            _ => {}
        }
    }
}

async fn run_server() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await.expect("Failed to bind");

    println!("Server listening on 127.0.0.1:8080");

    loop {
        let (socket, _) = listener.accept().await.expect("Failed to accept conneciton");

        // 新しい接続ごとにタスクを生成
        task::spawn(handle_connection(socket));
    }
}

#[tokio::main]
async fn main() {
    run_server().await;
}
