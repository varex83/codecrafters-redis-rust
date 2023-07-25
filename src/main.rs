use std::io::{BufRead, BufReader, BufWriter, Read, Write};
// Uncomment this block to pass the first stage
use std::net::TcpListener;
use bytes::buf::{Reader, Writer};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let mut reader = BufReader::new(_stream.try_clone().unwrap());

                let mut writer = BufWriter::new(_stream.try_clone().unwrap());


                loop {
                    let mut buffer = String::new();
                    let res = reader.read_line(&mut buffer);

                    if buffer.trim().is_empty() || res.is_err() {
                        println!("Closing connection");
                        break;
                    }

                    println!("Received: {}", buffer);

                    let res = writer.write_all(b"+PONG\r\n");

                    if res.is_err() {
                        println!("Closing connection");
                        break;
                    }

                    writer.flush().unwrap();

                    println!("Sent: +OK");
                }


            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
