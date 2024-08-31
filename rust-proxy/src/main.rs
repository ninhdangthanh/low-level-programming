use rustls_pemfile::{certs, read_one, Item};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::TlsAcceptor;
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::sync::Arc;
use std::fs::File;
use std::io::{self, BufReader, Read};


#[tokio::main]
async fn main() {
    // let certs = load_certs("keys/rootCA.crt");
    // let key = load_private_key("keys/rootCA.key");

    // let config = ServerConfig::builder()
    //         .with_safe_defaults()
    //         .with_no_client_auth()
    //         .with_single_cert(certs, key.unwrap()).expect("invalid key or certificate");

    // let acceptor = TlsAcceptor::from(Arc::new(config));
    
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

// async fn handle_client(mut stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>) {
//     let mut buf = [0u8; 1024];
    
//     tokio::spawn(async move {
//         {
//             match stream.read(&mut buf).await {
//                 Ok(n) if n == 0 => {
//                     println!("Client disconnected");
//                     return;
//                 }
//                 Ok(n) => {
//                     println!("Received from client: {:?}", &buf[..n]);
    
//                     if let Err(e) = stream.write_all(&buf[..n]).await {
//                         println!("Failed to write to client: {}", e);
//                     } else {
//                         println!("Sent response to client");
//                     }
//                 }
//                 Err(e) => {
//                     println!("Failed to read from client: {}", e);
//                     return;
//                 }
//             }
//         }
//     });
// }

// fn load_certs(path: &str) -> Vec<Certificate> {
//     let certfile = File::open(path).expect("Cannot open certificate file");
//     let mut reader = BufReader::new(certfile);

//     certs(&mut reader)
//         .unwrap()
//         .into_iter()
//         .map(Certificate)
//         .collect()
// }

// fn load_private_key(path: &str) -> io::Result<PrivateKey> {
//     let keyfile = File::open(path)?;
//     let mut reader = BufReader::new(keyfile);

//     while let Some(item) = read_one(&mut reader)? {
//         match item {
//             Item::RSAKey(key) | Item::PKCS8Key(key) => return Ok(PrivateKey(key)),
//             _ => continue,
//         }
//     }

//     Err(io::Error::new(io::ErrorKind::InvalidData, "No valid private key found"))
// }
