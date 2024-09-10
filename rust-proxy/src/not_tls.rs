use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Proxy server listening on 127.0.0.1:8080");

    loop {
        loop {
            let (stream, _addr) = listener.accept().await.unwrap();
    
            tokio::spawn(async move {
                if let Err(e) = handle_plain_client(stream).await {
                    eprintln!("Connection handling error: {:?}", e);
                }
            });
        }
    }
}

async fn handle_plain_client(mut stream: tokio::net::TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0u8; 1024];

    loop {
        match stream.read(&mut buf).await {
            Ok(n) => {
                println!("Received from client: {:?}", &buf[..n]);
                stream.write_all(&buf[..n]).await?;
            },
            Err(_) => {
                println!("Client disconnected");
            },
        }
    }
}