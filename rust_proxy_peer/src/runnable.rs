use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    println!("Proxy server listening on 127.0.0.1:8081");

    loop {
        let (stream, _addr) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                eprintln!("Connection handling error: {:?}", e);
            }
        });
    }
}

async fn handle_client(mut client_stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0u8; 1024];

    // Connect to the destination server
    let mut server_stream = TcpStream::connect("127.0.0.1:8080").await?;

    loop {
        // Read data from the client
        let n = client_stream.read(&mut buf).await?;
        if n == 0 {
            println!("Client disconnected");
            break;
        }
        println!("Received from client: {:?}", &buf[..n]);

        // Forward data to the destination server
        server_stream.write_all(&buf[..n]).await?;

        // Read response from the destination server
        let n = server_stream.read(&mut buf).await?;
        if n == 0 {
            println!("Server disconnected");
            break;
        }
        println!("Received from server: {:?}", &buf[..n]);

        // Send the response back to the client
        client_stream.write_all(&buf[..n]).await?;
    }

    Ok(())
}
