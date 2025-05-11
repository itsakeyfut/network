use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ChatRoom {
    pub name: String,
    pub users: RwLock<HashMap<String, String>>, // user_id -> username
    pub messages: RwLock<Vec<ChatMessage>>,
    pub max_messages: usize,
}

impl ChatRoom {
    pub fn new(name: String) -> Self {
        Self {
            name,
            users: RwLock::new(HashMap::new()),
            messages: RwLock::new(Vec::new()),
            max_messages: 100, // メッセージ履歴の最大数
        }
    }

    pub async fn add_user(&self, user_id: String, username: String) -> bool {
        let mut users = self.users.write().await;
        users.insert(user_id, username).is_none()
    }

    pub async fn remove_user(&self, user_id: &str) -> Option<String> {
        let mut users = self.users.write().await;
        users.remove(user_id)
    }

    pub async fn add_message(&self, message: ChatMessage) {
        let mut messages = self.messages.write().await;
        messages.push(message);
        
        // 最大メッセージ数を超えたら古いメッセージを削除
        if messages.len() > self.max_messages {
            messages.remove(0);
        }
    }
    

    pub async fn get_user_list(&self) -> Vec<String> {
        let users = self.users.read().await;
        users.values().cloned().collect()
    }

    pub async fn get_message_history(&self, last_n: usize) -> Vec<ChatMessage> {
        let messages = self.messages.read().await;
        let start = if messages.len() > last_n {
            messages.len() - last_n
        } else {
            0
        };
        messages[start..].to_vec()
    }
}