use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, rustls::ServerConfig};
use std::sync::Arc;
use std::io::{self, Error};
use futures::future::{self, TryFutureExt};
use std::fs;
use tokio::io::{copy_bidirectional, AsyncReadExt};

async fn handle_client(mut inbound: TcpStream, tls_acceptor: TlsAcceptor) -> io::Result<()> {
    // Perform the TLS handshake.
    let mut buf = vec![0; 1024];
    let readd = inbound.read(&mut buf).await.unwrap();
    println!("readd {:?}", readd);
    
    
    let tls_stream = tls_acceptor.accept(inbound).await?;
    
    // For simplicity, just echo the data back and forth.
    let target_addr: std::net::SocketAddr = "youtube.com:443".parse().unwrap();
    let mut outbound = TcpStream::connect(target_addr).await?;

    // Perform bidirectional data transfer between the client and the target server.
    let (mut tls_read, mut tls_write) = tokio::io::split(tls_stream);
    let (mut outbound_read, mut outbound_write) = tokio::io::split(outbound);

    let client_to_server = tokio::io::copy(&mut tls_read, &mut outbound_write);
    let server_to_client = tokio::io::copy(&mut outbound_read, &mut tls_write);

    future::try_join(client_to_server, server_to_client).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Load your TLS keys and certificates.
    let certs = load_certs("keys/cert.pem")?;
    let key = load_private_key("keys/key.pem")?;
    
    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let tls_acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind("0.0.0.0:8081").await?;
    println!("Proxy listening on 0.0.0.0:8081");

    loop {
        let (inbound, _) = listener.accept().await?;
        let tls_acceptor = tls_acceptor.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(inbound, tls_acceptor).await {
                eprintln!("Failed to handle client: {}", e);
            }
        });
    }
}

// Helper functions to load certificates and keys.
fn load_certs(path: &str) -> io::Result<Vec<rustls::Certificate>> {
    let certfile = fs::File::open(path).map_err(|e| Error::new(io::ErrorKind::NotFound, e))?;
    let mut reader = io::BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .map(|certs| certs.into_iter().map(rustls::Certificate).collect())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid certificate"))
}

fn load_private_key(path: &str) -> io::Result<rustls::PrivateKey> {
    let keyfile = fs::File::open(path).map_err(|e| Error::new(io::ErrorKind::NotFound, e))?;
    let mut reader = io::BufReader::new(keyfile);
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid private key"))?;
    if keys.len() != 1 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Expected a single private key"));
    }
    Ok(rustls::PrivateKey(keys[0].clone()))
}
