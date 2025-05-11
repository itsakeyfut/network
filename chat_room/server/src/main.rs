use server::server::ChatServer;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let server = ChatServer::new();
    server.run("127.0.0.1:8080").await?;

    Ok(())
}
