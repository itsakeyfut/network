use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sock = UdpSocket::bind("0.0.0.0:8080").await?;
    let mut buff = [0; 1024];

    loop {
        let (len, addr) = sock.recv_from(&mut buff).await?;
        println!("Received from {}: {:?}", addr, &buff[..len]);

        sock.send_to(b"ACK", &addr).await?;
    }
}
