use client::entity::message::{ClientMessage, ServerMessage};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use std::io;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (reader, mut writer) = stream.into_split();
    let reader = BufReader::new(reader);

    // ログイン
    print!("Enter your username: ");
    io::Write::flush(&mut io::stdout())?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    
    let login_msg = ClientMessage::Login { username: username.trim().to_string() };
    let json = serde_json::to_string(&login_msg)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    // 受信用タスク
    task::spawn(async move {
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(message) = serde_json::from_str::<ServerMessage>(&line.trim()) {
                match message {
                    ServerMessage::NewMessage { sender, content, .. } => {
                        println!("{}: {}", sender, content);
                    }
                    ServerMessage::UserJoined { username, room_name } => {
                        println!("*** {} joined {}", username, room_name);
                    }
                    ServerMessage::UserLeft { username, room_name } => {
                        println!("*** {} left {}", username, room_name);
                    }
                    _ => {
                        println!("{:?}", message);
                    }
                }
            }
        }
    });

    // 送信用ループ
    let mut input = String::new();
    loop {
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();

        let message = if trimmed.starts_with("/join ") {
            let room_name = trimmed[6..].to_string();
            ClientMessage::JoinRoom { room_name }
        } else if trimmed.starts_with("/create ") {
            let room_name = trimmed[8..].to_string();
            ClientMessage::CreateRoom { room_name }
        } else if trimmed == "/rooms" {
            ClientMessage::ListRooms
        } else if trimmed == "/users" {
            ClientMessage::ListUsers
        } else {
            ClientMessage::SendMessage { content: trimmed.to_string() }
        };

        let json = serde_json::to_string(&message)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;

        input.clear();
    }
}
