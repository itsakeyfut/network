use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sock = UdpSocket::bind("0.0.0.0:0").await?; // 任意の空いてるポートでバインド
    sock.connect("127.0.0.1:8080").await?;

    sock.send(b"Hello from UDP client").await?;

    let mut buf = [0u8; 1024];
    let n = sock.recv(&mut buf).await?;

    println!("Received: {}", String::from_utf8_lossy(&buf[..n]));

    Ok(())
}
