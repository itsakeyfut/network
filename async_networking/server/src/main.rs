use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buff = [0; 1024];

            match socket.read(&mut buff).await {
                Ok(n) if n == 0 => return,
                Ok(n) => {
                    if let Err(e) = socket.write_all(&buff[0..n]).await {
                        eprintln!("Failed to write: {}", e);
                    }
                }
                Err(e) => eprintln!("Failed to read: {}", e),
            }
        });
    }
}
