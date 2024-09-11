use clap::{App, Arg};
use utils::VpnPacket;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::thread;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use env_logger::Builder;
use log::LevelFilter;


mod client;
mod utils;



fn setup_tun_interface() -> Result<(), Box<dyn Error>> {
    let output = Command::new("sudo")
        .arg("ip")
        .arg("link")
        .arg("set")
        .arg("dev")
        .arg("tun2")
        .arg("up")
        .output()?;

    if !output.status.success() {
        return Err(format!("Failed to bring up tun2: {:?}", output.stderr).into());
    }

    let output = Command::new("sudo")
        .arg("ip")
        .arg("addr")
        .arg("add")
        .arg("10.8.0.2/24")
        .arg("dev")
        .arg("tun2")
        .output()?;

    if !output.status.success() {
        return Err(format!("Failed to assign IP to tun2: {:?}", output.stderr).into());
    }

    Ok(())
}

async fn destroy_tun_interface() {
    let output = Command::new("sudo")
        .arg("ip")
        .arg("link")
        .arg("delete")
        .arg("tun2")
        .output()
        .expect("Failed to execute command to delete TUN interface");

    if !output.status.success() {
        eprintln!("Failed to delete TUN interface: {}", String::from_utf8_lossy(&output.stderr));
    }
}

fn handle_client(client_id: usize, mut stream: TcpStream, clients: Arc<Mutex<HashMap<usize, TcpStream>>>) {
    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client {} disconnected", client_id);
                break;
            }
            Ok(n) => {
                let data = &buffer[0..n];

                println!("Server: data received from the client: {:?}", data);

                let mut clients_guard = clients.lock().unwrap();

                for (id, client_stream) in clients_guard.iter_mut() {
                    if *id != client_id {
                        let _ = client_stream.write(data);
                    }
                }
            }
            Err(e) => {
                println!("Error reading from client {}: {}", client_id, e);
                break;
            }
        }
    }

    clients.lock().unwrap().remove(&client_id);
    let _ = stream.shutdown(Shutdown::Both);
}

fn server_mode() {
    let listener = TcpListener::bind("0.0.0.0:12345").unwrap();
    let clients: Arc<Mutex<HashMap<usize, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));

    let mut config = tun::Configuration::default();
    config.name("tun2");
    let tun_device = tun::create(&config).unwrap();

    // Setup the tun2 interface
    if let Err(e) = setup_tun_interface() {
        eprintln!("Failed to set up TUN interface: {}", e);
        return;
    }

    let shared_tun = Arc::new(Mutex::new(tun_device));

    println!("Server started on 0.0.0.0:12345");

    let tun_device_clone = shared_tun.clone();
    let clients_clone = clients.clone();

    thread::spawn(move || {
        let clients_guard = clients_clone.lock().unwrap();

        if let Some(client) = clients_guard.get(&0) { //TODO: Implement multi-client
            if let Ok(client_clone) = client.try_clone() {
                drop(clients_guard);  // Unlock the mutex early
                let mut locked_tun = tun_device_clone.lock().unwrap();
                println!("Handle available client");
                read_from_tun_and_send_to_client(&mut *locked_tun, client_clone);
            } else {
                // Handle error while trying to clone the TcpStream
                println!("Failed to clone client TcpStream");
            }
        } else {
            // Handle the case where the client doesn't exist
            println!("No client with key 0 found");
        }
    });

    for (client_id, stream) in listener.incoming().enumerate() {
        match stream {
            Ok(stream) => {
                println!("New client connected with ID: {}", client_id);

                let tun_device_clone = shared_tun.clone();
                let clients_clone: Arc<Mutex<HashMap<usize, TcpStream>>> = clients.clone();

                thread::spawn(move || {
                    match clients_clone.lock().unwrap().get(&0) {
                        Some(client_clone_result) => {
                            let client_clone = client_clone_result.try_clone().unwrap();
                            let mut locked_tun = tun_device_clone.lock().unwrap();
                            read_from_tun_and_send_to_client(&mut *locked_tun, client_clone);
                        },
                        None => {
                            let size = {
                                let clients_guard = clients_clone.lock().unwrap();
                                clients_guard.len() 
                            };
                            println!("Not found user, current size of client is {}", size);
                        },
                    }
                });

                clients.lock().unwrap().insert(client_id, stream.try_clone().unwrap());
                let clients_arc = clients.clone();
                thread::spawn(move || handle_client(client_id, stream, clients_arc));
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }

    // Clean up the tun2 interface when done
    let _ = destroy_tun_interface();
}



fn read_from_tun_and_send_to_client<T: tun::Device>(tun: &mut T, mut client: TcpStream) {
    let mut buffer = [0u8; 1500];

    loop {
        match tun.read(&mut buffer) {
            Ok(n) => {
                match utils::encrypt(&buffer[..n]) {
                    Ok(encrypted_data) => {
                        println!("encrypted_data {:?}", encrypted_data);
                        // Handle sending the encrypted data to the client
                        println!("Received {} bytes from TUN device.", n);

                        let vpn_packet = VpnPacket { data: encrypted_data };
                        // Serialize and send to client
                        let serialized_data = bincode::serialize(&vpn_packet).unwrap();

                        match client.write_all(&serialized_data) {
                            Ok(_) => {
                                println!("SUCCESS: write all serialized data to client")
                            },
                            Err(_) => {
                                println!("FAILED: write all serialized data to client")
                            },
                        }
                    },
                    Err(err_msg) => {
                        // Handle the encryption error
                        println!("Encryption error: {}", err_msg);
                    }
                }
            },
            Err(e) => {
                // Handle the TUN reading error
                println!("TUN read error: {}", e);
            }
        }
    }

}


#[tokio::main]
async fn  main() {
    // Initialize the logger with 'info' as the default level
    Builder::new()
        .filter(None, LevelFilter::Info)
        .init();

    let matches = App::new("Simple VPN")
        .version("1.0")
        .author("Luis Soares")
        .about("A simple VPN tunnel in Rust")
        .arg(Arg::with_name("mode")
            .required(true)
            .index(1)
            .possible_values(&["server", "client"])
            .help("Runs the program in either server or client mode"))
        .arg(Arg::with_name("vpn-server")
            .long("vpn-server")
            .value_name("IP")
            .help("The IP address of the VPN server to connect to (client mode only)")
            .takes_value(true))
        .get_matches();

    let is_server_mode = matches.value_of("mode").unwrap() == "server";

    if is_server_mode {
        server_mode();
    } else {
        if let Some(vpn_server_ip) = matches.value_of("vpn-server") {
            let server_address = format!("{}:12345", vpn_server_ip);
            client::client_mode(server_address.as_str()).await;
        } else {
            eprintln!("Error: For client mode, you must provide the '--vpn-server' argument.");
        }
    }

}