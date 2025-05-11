use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;
use chrono::Utc;
use log::{info, error};

use crate::entity::message::{ClientMessage, ServerMessage};
use crate::room::{ChatMessage, ChatRoom};

#[derive(Debug)]
pub struct ChatServer {
    rooms: Arc<RwLock<HashMap<String, Arc<ChatRoom>>>>,
    users: Arc<RwLock<HashMap<String, User>>>,
    broadcast_tx: broadcast::Sender<(ServerMessage, Option<String>, Option<String>)>,
}

#[derive(Debug, Clone)]
struct User {
    id: String,
    username: String,
    current_room: Option<String>,
    tx: broadcast::Sender<ServerMessage>,
}

impl ChatServer {
    pub fn new() -> Self {
        let (broadcast_tx, _rx) = broadcast::channel(1000);

        let mut rooms = HashMap::new();
        let general_room = Arc::new(ChatRoom::new("general".to_string()));
        rooms.insert("general".to_string(), general_room);

        Self {
            rooms: Arc::new(RwLock::new(rooms)),
            users: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }

    pub async fn run(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        info!("Chat server listening on {}", addr);

        loop {
            let (socket, addr) = listener.accept().await?;
            info!("New connection from: {}", addr);

            let server = Arc::new(self.clone());
            tokio::spawn(async move {
                if let Err(e) = server.handle_client(socket).await {
                    error!("Error handling client {}: {}", addr, e);
                }
            });
        }
    }

    pub async fn handle_client(&self, stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        // ユーザーの初期化
        let mut user_id: Option<String> = None;
        let mut user_rx: Option<broadcast::Receiver<ServerMessage>> = None;

        // 受信ループ
        loop {
            tokio::select! {
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            // 接続が閉じられた
                            if let Some(uid) = &user_id {
                                self.handle_user_disconnect(uid).await;
                            }
                            break;
                        }
                        Ok(_) => {
                            if let Ok(message) = serde_json::from_str::<ClientMessage>(&line.trim()) {
                                match message {
                                    ClientMessage::Login { username } => {
                                        let uid = Uuid::new_v4().to_string();
                                        let (tx, rx) = broadcast::channel(100);

                                        let user = User {
                                            id: uid.clone(),
                                            username: username.clone(),
                                            current_room: Some("general".to_string()),
                                            tx: tx.clone(),
                                        };

                                        // ユーザーを追加
                                        {
                                            let mut users = self.users.write().await;
                                            users.insert(uid.clone(), user);
                                        }

                                        // 一般ルームに追加
                                        {
                                            let rooms = self.rooms.read().await;
                                            if let Some(room) = rooms.get("general") {
                                                room.add_user(uid.clone(), username.clone()).await;
                                            }
                                        }

                                        user_id = Some(uid.clone());
                                        user_rx = Some(rx);

                                        // ウェルカムメッセージ
                                        let welcome_msg = ServerMessage::Welcome { user_id: uid.clone() };
                                        self.send_message(welcome_msg.clone(), Some(uid.clone()), None).await;

                                        // ルーム参加追加
                                        let join_msg = ServerMessage::UserJoined {
                                            username: username.clone(),
                                            room_name: "general".to_string()
                                        };
                                        self.send_message(join_msg, None, Some("general".to_string())).await;

                                        info!("User {} logged in", username);
                                    }
                                    _ => {
                                        if let Some(uid) = &user_id {
                                            self.handle_message(uid.clone(), message).await;
                                        }
                                    }
                                }
                            }
                            line.clear();
                        }
                        Err(e) => {
                            error!("Error reading line: {}", e);
                            break;
                        }
                    }
                }

                result = async {
                    if let Some(rx) = &mut user_rx {
                        rx.recv().await
                    } else {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        Err(broadcast::error::RecvError::Lagged(0))
                    }
                } => {
                    if let Ok(message) = result {
                        let json = serde_json::to_string(&message)?;
                        writer.write_all(json.as_bytes()).await?;
                        writer.write_all(b"\n").await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_message(&self, user_id: String, message: ClientMessage) {
        let users = self.users.read().await;
        let user = match users.get(&user_id) {
            Some(user) => user.clone(),
            None => return,
        };
        drop(users);

        match message {
            ClientMessage::SendMessage { content } => {
                if let Some(room_name) = &user.current_room {
                    let rooms = self.rooms.read().await;
                    if let Some(room) = rooms.get(room_name) {
                        let chat_message = ChatMessage {
                            sender: user.username.clone(),
                            content: content.clone(),
                            timestamp: Utc::now(),
                        };

                        room.add_message(chat_message).await;

                        let server_message = ServerMessage::NewMessage {
                            sender: user.username.clone(),
                            content,
                            room_name: room_name.clone(),
                            timestamp: Utc::now().to_rfc3339(),
                        };

                        self.send_message(server_message, None, Some(room_name.clone())).await;
                    }
                }
            }

            ClientMessage::CreateRoom { room_name } => {
                let mut rooms = self.rooms.write().await;
                if !rooms.contains_key(&room_name) {
                    let new_room = Arc::new(ChatRoom::new(room_name.clone()));
                    rooms.insert(room_name.clone(), new_room);

                    let response = ServerMessage::RoomCreated { room_name };
                    self.send_message(response, Some(user_id), None).await;
                } else {
                    let error_msg = ServerMessage::Error {
                        message: "Room already exists".to_string()
                    };
                    self.send_message(error_msg, Some(user_id), None).await;
                }
            }

            ClientMessage::JoinRoom { room_name } => {
                let rooms = self.rooms.read().await;
                if let Some(room) = rooms.get(&room_name) {
                    // 現在のルームから離脱
                    if let Some(current_room_name) = &user.current_room {
                        if let Some(current_room) = rooms.get(current_room_name) {
                            if let Some(username) = current_room.remove_user(&user_id).await {
                                let leave_msg = ServerMessage::UserLeft {
                                    username: username.clone(),
                                    room_name: current_room_name.clone(),
                                };
                                self.send_message(leave_msg, None, Some(current_room_name.clone())).await;
                            }
                        }
                    }

                    // 新しいルームに追加
                    room.add_user(user_id.clone(), user.username.clone()).await;

                    // ユーザーの現在ルームを更新
                    {
                        let mut users = self.users.write().await;
                        if let Some(u) = users.get_mut(&user_id) {
                            u.current_room = Some(room_name.clone());
                        }
                    }

                    // 参加通知
                    let join_msg = ServerMessage::UserJoined {
                        username: user.username.clone(),
                        room_name: room_name.clone(),
                    };
                    self.send_message(join_msg, None, Some(room_name.clone())).await;

                    // 参加確認をユーザーに送信
                    let joined_msg = ServerMessage::JoinedRoom { room_name };
                    self.send_message(joined_msg, Some(user_id), None).await;
                } else {
                    let error_msg = ServerMessage::Error {
                        message: "Room not found".to_string()
                    };
                    self.send_message(error_msg, Some(user_id), None).await;
                }
            }

            ClientMessage::ListRooms => {
                let rooms = self.rooms.read().await;
                let room_names: Vec<String> = rooms.keys().cloned().collect();
                let response = ServerMessage::RoomList { rooms: room_names };
                self.send_message(response, Some(user_id), None).await;
            }

            ClientMessage::ListUsers => {
                if let Some(room_name) = &user.current_room {
                    let rooms = self.rooms.read().await;
                    if let Some(room) = rooms.get(room_name) {
                        let user_list = room.get_user_list().await;
                        let response = ServerMessage::UserList { users: user_list };
                        self.send_message(response, Some(user_id), None).await;
                    }
                }
            }

            _ => {}
        }
    }

    async fn send_message(&self, message: ServerMessage, target_user_id: Option<String>, target_room_name: Option<String>) {
        if let Err(e) = self.broadcast_tx.send((message, target_user_id, target_room_name)) {
            error!("Failed to send broadcast message: {}", e);
        }
    }

    async fn handle_user_disconnect(&self, user_id: &str) {
        let mut users = self.users.write().await;
        if let Some(user) = users.remove(user_id) {
            info!("User {} disconnected", user.username);

            // 現在のルームから離脱
            if let Some(room_name) = &user.current_room {
                let rooms = self.rooms.read().await;
                if let Some(room) = rooms.get(room_name) {
                    room.remove_user(user_id).await;

                    let leave_msg = ServerMessage::UserLeft {
                        username: user.username.clone(),
                        room_name: room_name.clone(),
                    };
                    self.send_message(leave_msg, None, Some(room_name.clone())).await;
                }
            }
        }
    }
}

impl Clone for ChatServer {
    fn clone(&self) -> Self {
        Self {
            rooms: Arc::clone(&self.rooms),
            users: Arc::clone(&self.users),
            broadcast_tx: self.broadcast_tx.clone(),
        }
    }
}