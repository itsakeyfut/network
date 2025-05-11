use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match TcpStream::connect("127.0.0.1:8080").await {
        Ok(mut stream) => {
            if let Err(e) = stream.write_all(b"Hello Async Server").await {
                eprintln!("Failed to write to server: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error>);
            }

            let mut buff = vec![0; 1024];
            match stream.read(&mut buff).await {
                Ok(n) => {
                    println!("Received: {}", String::from_utf8_lossy(&buff[..n]));
                }
                Err(e) => {
                    eprintln!("Failed to read from server: {}", e);
                    return Err(Box::new(e) as Box<dyn std::error::Error>);
                }
            }
        }
        Err(e) => {
            eprintln!("Couldn't connect to server: {}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    }

    Ok(())
}
