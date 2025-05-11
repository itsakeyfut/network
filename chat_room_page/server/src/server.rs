use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use log::info;

use crate::entity::message::{ClientMessage, ServerMessage};
use crate::room::{ChatMessage, ChatRoom};

#[derive(Debug)]
pub struct ChatServer {
    rooms: Arc<RwLock<HashMap<String, Arc<ChatRoom>>>>,
    users: Arc<RwLock<HashMap<String, User>>>,
    message_queues: Arc<RwLock<HashMap<String, VecDeque<ServerMessage>>>>,
}

#[derive(Debug, Clone)]
struct User {
    id: String,
    username: String,
    current_room: Option<String>,
}

impl ChatServer {
    pub fn new() -> Self {
        let mut rooms = HashMap::new();
        let general_room = Arc::new(ChatRoom::new("general".to_string()));
        rooms.insert("general".to_string(), general_room);

        Self {
            rooms: Arc::new(RwLock::new(rooms)),
            users: Arc::new(RwLock::new(HashMap::new())),
            message_queues: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub async fn register_user(&mut self, user_id: String, username: String) {
        let user = User {
            id: user_id.clone(),
            username: username.clone(),
            current_room: Some("general".to_string()),
        };

        // ユーザーを追加
        {
            let mut users = self.users.write().await;
            users.insert(user_id.clone(), user);
        }

        // メッセージキューを作成
        {
            let mut queues = self.message_queues.write().await;
            queues.insert(user_id.clone(), VecDeque::new());
        }

        // 一般ルームに追加
        {
            let rooms = self.rooms.read().await;
            if let Some(room) = rooms.get("general") {
                room.add_user(user_id.clone(), username.clone()).await;
            }
        }

        // ルーム参加通知
        let join_msg = ServerMessage::UserJoined {
            username: username.clone(),
            room_name: "general".to_string()
        };
        self.broadcast_room_message("general".to_string(), join_msg).await;

        info!("User {} logged in", username);
    }

    pub async fn handle_message(&mut self, user_id: String, message: ClientMessage) {
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
                        
                        self.broadcast_room_message(room_name.clone(), server_message).await;
                    }
                }
            }
            
            ClientMessage::CreateRoom { room_name } => {
                let mut rooms = self.rooms.write().await;
                if !rooms.contains_key(&room_name) {
                    let new_room = Arc::new(ChatRoom::new(room_name.clone()));
                    rooms.insert(room_name.clone(), new_room);
                    
                    let response = ServerMessage::RoomCreated { room_name };
                    self.send_direct_message(user_id, response).await;
                } else {
                    let error_msg = ServerMessage::Error { 
                        message: "Room already exists".to_string() 
                    };
                    self.send_direct_message(user_id, error_msg).await;
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
                                self.broadcast_room_message(current_room_name.clone(), leave_msg).await;
                            }
                        }
                    }
                    
                    // 新しいルームに参加
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
                    self.broadcast_room_message(room_name.clone(), join_msg).await;
                    
                    // 参加確認をユーザーに送信
                    let joined_msg = ServerMessage::JoinedRoom { room_name };
                    self.send_direct_message(user_id, joined_msg).await;
                } else {
                    let error_msg = ServerMessage::Error { 
                        message: "Room not found".to_string() 
                    };
                    self.send_direct_message(user_id, error_msg).await;
                }
            }
            
            ClientMessage::ListRooms => {
                let rooms = self.rooms.read().await;
                let room_names: Vec<String> = rooms.keys().cloned().collect();
                let response = ServerMessage::RoomList { rooms: room_names };
                self.send_direct_message(user_id, response).await;
            }
            
            ClientMessage::ListUsers => {
                if let Some(room_name) = &user.current_room {
                    let rooms = self.rooms.read().await;
                    if let Some(room) = rooms.get(room_name) {
                        let user_list = room.get_user_list().await;
                        let response = ServerMessage::UserList { users: user_list };
                        self.send_direct_message(user_id, response).await;
                    }
                }
            }
            
            _ => {}
        }
    }

    async fn send_direct_message(&self, user_id: String, message: ServerMessage) {
        let mut queues = self.message_queues.write().await;
        if let Some(queue) = queues.get_mut(&user_id) {
            queue.push_back(message);
        }
    }

    async fn broadcast_room_message(&self, room_name: String, message: ServerMessage) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(&room_name) {
            let users = room.users.read().await;
            let user_ids: Vec<String> = users.keys().cloned().collect();

            let mut queues = self.message_queues.write().await;
            for user_id in user_ids {
                if let Some(queue) = queues.get_mut(&user_id) {
                    queue.push_back(message.clone());
                }
            }
        }
    }

    pub async fn get_pending_messages(&mut self, user_id: &str) -> Vec<ServerMessage> {
        let mut queues = self.message_queues.write().await;
        if let Some(queue) = queues.get_mut(user_id) {
            let messages: Vec<ServerMessage> = queue.drain(..).collect();
            return messages;
        }
        Vec::new()
    }

    pub async fn handle_user_disconnect(&mut self, user_id: &str) {
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
                    self.broadcast_room_message(room_name.clone(), leave_msg).await;
                }
            }
            
            // メッセージキューを削除
            let mut queues = self.message_queues.write().await;
            queues.remove(user_id);
        }
    }
}