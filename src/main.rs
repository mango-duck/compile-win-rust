use std::time::SystemTime;
use chrono::{DateTime, Local};

fn main() {
    // 获取当前时间
    let now = SystemTime::now();
    let datetime: DateTime<Local> = now.into();
    
    // 打印欢迎信息
    println!("====================================");
    println!("Rust Windows 程序示例");
    println!("编译时间: {}", datetime.format("%Y-%m-%d %H:%M:%S"));
    println!("系统信息:");
    println!("  - 目标架构: {}", std::env::consts::ARCH);
    println!("  - 操作系统: {}", std::env::consts::OS);
    println!("====================================");
    
    // 简单的计算示例
    let result = add(3, 5);
    println!("3 + 5 = {}", result);
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
        assert_eq!(add(-1, 5), 4);
    }
}
