use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_native_tls::native_tls::Certificate;
use tokio_rustls::{TlsConnector, rustls::{ClientConfig, RootCertStore}};
use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

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

    // Load the SSL certificate and private key
    let cert_file = &mut BufReader::new(File::open("my_cert.pem")?);
    let key_file = &mut BufReader::new(File::open("my_key.pem")?);
    
    let certs = rustls_pemfile::certs(cert_file)?.into_iter().map(Certificate).collect();
    let mut keys = rustls_pemfile::pkcs8_private_keys(key_file)?;

    // Setup ClientConfig
    let mut config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(RootCertStore::empty())
        .with_single_cert(certs, PrivateKey(keys.remove(0)))?;
    
    let connector = TlsConnector::from(Arc::new(config));

    // Connect to the destination server with TLS
    let domain = DNSNameRef::try_from_ascii_str("youtube.com").unwrap();
    let server_stream = TcpStream::connect("youtube.com:443").await?;
    let mut server_stream = connector.connect(domain, server_stream).await?;

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
