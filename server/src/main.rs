// server.rs
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

type ClientMap = Arc<Mutex<HashMap<String, TcpStream>>>;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").expect("Failed to bind");
    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    println!("Server listening on 0.0.0.0:7878");

    // 接受客户端连接
    thread::spawn({
        let clients = Arc::clone(&clients);
        move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let addr = stream.peer_addr().unwrap().to_string();
                        println!("New client connected: {}", addr);
                        
                        let clients = Arc::clone(&clients);
                        clients.lock().unwrap().insert(addr.clone(), stream.try_clone().unwrap());
                        
                        thread::spawn(move || {
                            handle_client(stream, addr, clients);
                        });
                    }
                    Err(e) => {
                        println!("Connection failed: {}", e);
                    }
                }
            }
        }
    });

    // 主线程处理用户输入
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "list" {
            let clients = clients.lock().unwrap();
            println!("Connected clients:");
            for (addr, _) in clients.iter() {
                println!("- {}", addr);
            }
            continue;
        }

        // 命令格式: <client_addr> <command>
        if let Some((addr, cmd)) = input.split_once(' ') {
            let mut clients = clients.lock().unwrap();
            if let Some(stream) = clients.get_mut(addr) {
                if let Err(e) = stream.write_all(cmd.as_bytes()) {
                    println!("Failed to send command to {}: {}", addr, e);
                    clients.remove(addr);
                }
            } else {
                println!("Client {} not found", addr);
            }
        } else {
            println!("Usage: <client_addr> <command>");
            println!("Commands:");
            println!("  list - List connected clients");
        }
    }
}

fn handle_client(mut stream: TcpStream, addr: String, clients: ClientMap) {
    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                let output = String::from_utf8_lossy(&buffer[..size]);
                println!("[{}] Output:\n{}", addr, output);
            }
            Ok(_) => break, // 连接关闭
            Err(e) => {
                println!("Error with client {}: {}", addr, e);
                break;
            }
        }
    }

    // 客户端断开连接，从map中移除
    clients.lock().unwrap().remove(&addr);
    println!("Client {} disconnected", addr);
}
