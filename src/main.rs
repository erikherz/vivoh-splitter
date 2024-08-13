use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::env;

fn main() -> std::io::Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let bind_address = args.iter().position(|x| x == "--bind")
        .map(|i| args[i + 1].clone())
        .unwrap_or("0.0.0.0:5005".to_string());
    let key = args.iter().position(|x| x == "--key")
        .map(|i| args[i + 1].clone())
        .unwrap_or_else(|| "".to_string());

    // Bind the server to the specified address and port
    let server_socket = UdpSocket::bind(&bind_address)?;
    server_socket.set_nonblocking(true)?;

    println!("Listening for UDP traffic on {}", bind_address);

    // Shared list of clients
    let clients = Arc::new(Mutex::new(Vec::new()));
    let clients_clone = Arc::clone(&clients);
    let server_socket_clone = server_socket.try_clone()?;

    // Thread to manage client registration
    thread::spawn(move || {
        let mut buf = [0; 1024];

        loop {
            if let Ok((size, addr)) = server_socket_clone.recv_from(&mut buf) {
                let received_key = String::from_utf8_lossy(&buf[..size]).to_string();

                // Register the client if the key matches
                if received_key.trim() == key {
                    let mut clients = clients_clone.lock().unwrap();
                    if !clients.contains(&addr) {
                        clients.push(addr);
                        println!("Client {} subscribed with key {}", addr, key);
                    }
                } else {
                    println!("Client {} provided invalid key: {}", addr, received_key);
                }
            }
        }
    });

    // Main loop to forward packets to registered clients
    let mut buf = [0; 1024];
    loop {
        if let Ok((size, _)) = server_socket.recv_from(&mut buf) {
            let clients = clients.lock().unwrap();
            for &client in clients.iter() {
                server_socket.send_to(&buf[..size], client)?;
            }
        }
    }
}

