use std::io::{BufRead, BufReader, BufWriter, Read, Write};
// Uncomment this block to pass the first stage
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let mut buffer = [0u8; 512];
                loop {
                    let res = _stream.read(&mut buffer).unwrap();

                    if res == 0 {
                        println!("Closing connection");
                        break;
                    }

                    let res = _stream.write(b"+PONG\r\n");

                    if res.is_err() {
                        println!("Closing connection");
                        break;
                    }

                    println!("Sent: +OK");
                }


            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
