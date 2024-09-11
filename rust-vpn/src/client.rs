use std::net::TcpStream;
use std::io::{Read, Write};
use std::process::Command;
use tun::platform::Device;
use log::error;

use crate::utils::{decrypt, VpnPacket};



const TUN_INTERFACE_NAME: &str = "tun3";


fn set_client_ip_and_route() {
    let ip_output = Command::new("ip")
        .arg("addr")
        .arg("add")
        .arg("10.8.0.3/24")
        .arg("dev")
        .arg("tun3")
        .output()
        .expect("Failed to execute IP command");

    if !ip_output.status.success() {
        eprintln!("Failed to set IP: {}", String::from_utf8_lossy(&ip_output.stderr));
        return;
    }

    let link_output = Command::new("ip")
        .arg("link")
        .arg("set")
        .arg("up")
        .arg("dev")
        .arg("tun3")
        .output()
        .expect("Failed to execute IP LINK command");

    if !link_output.status.success() {
        eprintln!("Failed to set link up: {}", String::from_utf8_lossy(&link_output.stderr));
        return;
    }

    let route_output = Command::new("ip")
        .arg("route")
        .arg("add")
        .arg("0.0.0.0/0")
        .arg("via")
        .arg("10.8.0.2")
        .arg("dev")
        .arg("tun3")
        .output()
        .expect("Failed to execute IP ROUTE command");

    if !route_output.status.success() {
        eprintln!("Failed to set route: {}", String::from_utf8_lossy(&route_output.stderr));
    }
}

async fn read_from_client_and_write_to_tun(client: &mut TcpStream, tun: &mut Device) {
    let mut buffer = [0u8; 1500];
    loop {
        match client.read(&mut buffer) {
            Ok(n) => {
                let vpn_packet: VpnPacket = bincode::deserialize(&buffer[..n]).unwrap();
                let decrypted_data = decrypt(&vpn_packet.data);

                println!("Writing data to tun3, {:?}", decrypted_data);

                // tun.write(&decrypted_data).unwrap();
            }
            Err(e) => {
                error!("Error reading from client: {}", e);
                continue;
            }
        };


    }
}


pub async fn client_mode(vpn_server_ip: &str) {
    // Basic client mode for demonstration
    let mut stream = TcpStream::connect(vpn_server_ip).unwrap();

    // Clone the stream we can use it both inside and outside the async block
    let mut stream_clone = stream.try_clone().unwrap();

    let mut config = tun::Configuration::default();
    config.name(TUN_INTERFACE_NAME);
    let mut tun_device = tun::create(&config).unwrap();

    // Set the client's IP and routing
    set_client_ip_and_route();

    println!("Connected to the server {}", vpn_server_ip);

    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                println!("{} Received from the server", n);

                println!("About to call read_from_client_and_write_to_tun");

                read_from_client_and_write_to_tun(&mut stream_clone, &mut tun_device).await;

                println!("Finished calling read_from_client_and_write_to_tun");
            }
            Err(_) => {
                break;
            }
        }
    }
}