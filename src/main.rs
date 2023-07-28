mod parser;

use crate::parser::Token::Command;
use crate::parser::{CommandIdent, Parser, Token};
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (stream, addr) = listener.accept().await.unwrap();

        println!("Got new connection from {:?}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                println!("an error occurred; error = {:?}", e);
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0u8; 512];
    loop {
        let res = stream.read(&mut buffer).await?;

        if res == 0 {
            println!("Closing connection");
            break;
        }

        let parser = Parser::new(String::from_utf8(buffer[..res].to_vec()).unwrap());

        let result = Vec::<String>::new();
        for token in parser {
            match token {
                Token::Array(len, tokens) => {
                    println!("Got array: {:?}", tokens);

                    let command = tokens[0].clone();

                    match command {
                        Command(CommandIdent::Ping) => {
                            stream.write(b"+PONG\r\n").await?;
                        }
                        Command(CommandIdent::Echo) => {
                            let message = tokens[1].clone();

                            stream.write(format!("+{}\r\n", message).as_bytes()).await?;
                        }
                        _ => {
                            println!("Got token: {:?}", tokens);

                            stream.write(b"+PONG\r\n").await?;
                        }
                    }
                }
                _ => {
                    println!("Got token: {:?}", token);
                }
            }
        }

        // stream.write(b"+PONG\r\n").await?;

        println!("Sent: +OK");
    }

    Ok(())
}
