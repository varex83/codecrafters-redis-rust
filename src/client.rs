use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <ip> <port>", args[0]);
        return;
    }

    let ip = &args[1];
    let port = &args[2];

    let mut stream = TcpStream::connect(format!("{}:{}", ip, port)).unwrap();

    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let mut tokens = line
            .split_whitespace()
            .fold(String::new(), |acc, x| acc.to_owned() + "\r\n" + x);

        tokens.push_str("\r\n");
        tokens = tokens.trim().to_string();

        stream.write_all(tokens.as_bytes()).unwrap();

        let mut buf = [0u8; 256];
        stream.read(&mut buf).unwrap();
        let buf = String::from_utf8(buf.to_vec()).unwrap();

        println!("\x1b[32m{}\x1b[0m", buf);
    }
}
