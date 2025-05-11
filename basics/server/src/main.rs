use std::net::TcpListener;
use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                stream.read(&mut buffer)?;
                stream.write_all(b"Hello from server")?;
            }
            Err(e) => println!("Error: {}", e),
        }
    }
    Ok(())
}
