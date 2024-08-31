use tokio::io::{copy, split};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task;

async fn transfer_data(mut reader: TcpStream, mut writer: TcpStream) {
    let (mut reader_rx, mut writer_tx) = split(reader);
    let (mut writer_rx, mut reader_tx) = split(writer);

    let _ = tokio::join!(
        copy(&mut reader_rx, &mut reader_tx),
        copy(&mut writer_rx, &mut writer_tx),
    );
}

async fn handle_connection(client_stream: TcpStream, target_addr: String) {
    match TcpStream::connect(&target_addr).await {
        Ok(server_stream) => {
            transfer_data(client_stream, server_stream).await;
        }
        Err(e) => eprintln!("Failed to connect to target server: {}", e),
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let target_addr = "127.0.0.1:8081".to_string();

    println!("Proxy server listening on 127.0.0.1:8080");

    let (stream, _addr) = listener.accept().await.unwrap();

    let mut peek_buf = [0u8; 16384];
    loop {
        match stream.peek(&mut peek_buf).await {
            Ok(_) => {
                if peek_buf[0] == 0x05 {
                    println!("sock5 request")
                } else {
                    let peer = stream.peer_addr();
                    println!("peer {:?}", peer);
                }
            },
            Err(_) => {
                println!("error picking first bytes")
            },
        }
    }

}
