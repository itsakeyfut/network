use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Login { username: String },
    SendMessage { content: String },
    JoinRoom { room_name: String },
    LeaveRoom { room_name: String },
    CreateRoom { room_name: String },
    ListRooms,
    ListUsers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Welcome { user_id: String },
    UserJoined { username: String, room_name: String },
    UserLeft { username: String, room_name: String },
    NewMessage { sender: String, content: String, room_name: String, timestamp: String },
    RoomCreated { room_name: String },
    JoinedRoom { room_name: String },
    LeftRoom { room_name: String },
    RoomList { rooms: Vec<String> },
    UserList { users: Vec<String> },
    Error { message: String }
}