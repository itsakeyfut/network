use std::sync::Arc;

use actix_web::{http, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actix::{Actor, ActorContext, StreamHandler};
use actix::AsyncContext;
use tokio::sync::Mutex;
use log::info;

use entity::message::{ClientMessage, ServerMessage};
use server::ChatServer;

mod entity;
mod room;
mod server;

struct WsSession {
    user_id: Option<String>,
    server: Arc<Mutex<ChatServer>>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // JSONメッセージをパース
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    let server = self.server.clone();
                    let actor_addr = ctx.address();
                    let current_id = self.user_id.clone(); // 現在のユーザーIDを取得
                    
                    // 非同期でメッセージを処理
                    actix::spawn(async move {
                        let mut server = server.lock().await;
                        
                        match &client_msg {
                            ClientMessage::Login { username } => {
                                // 新規ユーザー登録
                                let uid = uuid::Uuid::new_v4().to_string();
                                
                                server.register_user(uid.clone(), username.clone()).await;
                                
                                // ウェルカムメッセージを送信
                                let welcome = ServerMessage::Welcome { user_id: uid.clone() };
                                let json = serde_json::to_string(&welcome).unwrap();
                                actor_addr.do_send(WsMessage(json));
                                
                                // ユーザーIDをセッションに保存
                                actor_addr.do_send(SetUserId(uid.clone()));
                                
                                server.handle_message(uid.clone(), client_msg).await;
                            }
                            _ => {
                                // すでにログイン済みの場合は、保存されたユーザーIDを使用
                                if let Some(uid) = &current_id {
                                    server.handle_message(uid.clone(), client_msg).await;
                                    
                                    // 保留中のメッセージを取得して送信
                                    let messages = server.get_pending_messages(uid).await;
                                    for server_msg in messages {
                                        let json = serde_json::to_string(&server_msg).unwrap();
                                        actor_addr.do_send(WsMessage(json));
                                    }
                                }
                            }
                        }
                    });
                }
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Binary(_)) => {}
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

// ユーザーIDを設定するメッセージ
struct SetUserId(String);

impl actix::Message for SetUserId {
    type Result = ();
}

impl actix::Handler<SetUserId> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: SetUserId, _: &mut Self::Context) {
        self.user_id = Some(msg.0);
    }
}

// WebSocketにメッセージを送信するためのメッセージ型
struct WsMessage(String);

impl actix::Message for WsMessage {
    type Result = ();
}

impl actix::Handler<WsMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Arc<Mutex<ChatServer>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let session = WsSession {
        user_id: None,
        server: server.get_ref().clone(),
    };

    ws::start(session, &req, stream)
}

async fn get_rooms(server: web::Data<Arc<Mutex<ChatServer>>>) -> HttpResponse {
    let server = server.get_ref().clone();
    let server = server.lock().await;
    let rooms = server.get_room_list().await;

    HttpResponse::Ok().json(rooms)
}

// 接続が切断されたときのハンドラ
impl Drop for WsSession {
    fn drop(&mut self) {
        if let Some(uid) = &self.user_id {
            let server = self.server.clone();
            let uid = uid.clone();

            actix::spawn(async move {
                let mut server = server.lock().await;
                server.handle_user_disconnect(&uid).await;
            });
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    // チャットサーバーの初期化
    let chat_server = Arc::new(Mutex::new(ChatServer::new()));
    let server_data = web::Data::new(chat_server);

    let backend_port = 8080;
    let frontend_url = "http://localhost:3000";
    
    info!("Starting server at http:localhost:{}", backend_port);
    
    HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone())
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("Access-Control-Allow-Origin", frontend_url))
            )
            .wrap(
                actix_cors::Cors::default()
                    .allowed_origin(frontend_url)
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600)
            )
            .service(web::resource("/ws").to(ws_route))
            .service(web::resource("/api/rooms").route(web::get().to(get_rooms)))
    })
    .bind(("127.0.0.1", backend_port))?
    .run()
    .await
}