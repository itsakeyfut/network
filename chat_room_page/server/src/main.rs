use std::sync::Arc;
use server::server::ChatServer;
use warp::Filter;
use log::info;
use tokio::sync::Mutex;


#[tokio::main]
async fn main() {
    env_logger::init();
    
    // チャットサーバーの初期化
    let chat_server = Arc::new(Mutex::new(ChatServer::new()));
    
    // WebSocketハンドラ
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_chat_server(chat_server.clone()))
        .map(|ws: warp::ws::Ws, server: Arc<Mutex<ChatServer>>| {
            ws.on_upgrade(move |socket| handle_websocket(socket, server))
        });
    
    // 静的ファイル配信
    let static_files = warp::path("static")
        .and(warp::fs::dir("static"));
    
    // ルートパスでのindex.html配信
    let index = warp::path::end()
        .and(warp::fs::file("static/index.html"));
    
    let routes = ws_route
        .or(static_files)
        .or(index)
        .with(warp::cors().allow_any_origin());
    
    info!("Starting server at http://localhost:8080");
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

fn with_chat_server(
    chat_server: Arc<Mutex<ChatServer>>
) -> impl Filter<Extract = (Arc<Mutex<ChatServer>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || chat_server.clone())
}

async fn handle_websocket(ws: warp::ws::WebSocket, server: Arc<Mutex<ChatServer>>) {
    use futures::{SinkExt, StreamExt};
    use server::entity::message::{ClientMessage, ServerMessage};
    use warp::ws::Message;

    // WebSocketストリームを分割
    let (mut ws_tx, mut ws_rx) = ws.split();
    
    // ユーザーIDの初期化
    let mut user_id: Option<String> = None;
    
    // WebSocketからメッセージを受信
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(text) {
                        let mut server = server.lock().await;
                        
                        match &client_msg {
                            ClientMessage::Login { username } => {
                                // 新規ユーザー登録
                                let uid = uuid::Uuid::new_v4().to_string();
                                user_id = Some(uid.clone());
                                
                                server.register_user(uid.clone(), username.clone()).await;
                                
                                // ウェルカムメッセージを送信
                                let welcome = ServerMessage::Welcome { user_id: uid.clone() };
                                let json = serde_json::to_string(&welcome).unwrap();
                                if let Err(e) = ws_tx.send(Message::text(json)).await {
                                    eprintln!("Error sending welcome message: {}", e);
                                    break;
                                }
                                
                                server.handle_message(uid.clone(), client_msg).await;
                            }
                            _ => {
                                if let Some(uid) = &user_id {
                                    server.handle_message(uid.clone(), client_msg).await;
                                }
                            }
                        }
                        
                        // サーバーからのメッセージを処理
                        if let Some(uid) = &user_id {
                            let messages = server.get_pending_messages(uid).await;
                            for server_msg in messages {
                                let json = serde_json::to_string(&server_msg).unwrap();
                                if let Err(e) = ws_tx.send(Message::text(json)).await {
                                    eprintln!("Error sending message: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }
    
    // 接続が切断された場合のクリーンアップ
    if let Some(uid) = user_id {
        let mut server = server.lock().await;
        server.handle_user_disconnect(&uid).await;
    }
}