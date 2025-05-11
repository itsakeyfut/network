use std::net::TcpStream;
use std::io::{Read, Write};

fn main() {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {
        stream.write_all(b"Ping").unwrap();

        let mut buff = [0; 1024];
        let n = stream.read(&mut buff).unwrap();
        println!("Received: {}", String::from_utf8_lossy(&buff[..n]));
    } else {
        println!("Couldn't connect to server...");
    }
}
