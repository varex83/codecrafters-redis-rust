mod parser;

use crate::parser::Token::Command;
use crate::parser::{CommandIdent, Parser, Token};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db = HashMap::<Token, Token>::new();
    let db_rw = Arc::new(Mutex::new(db));

    loop {
        let (stream, addr) = listener.accept().await.unwrap();

        println!("Got new connection from {:?}", addr);

        let db_rw = db_rw.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, db_rw).await {
                println!("an error occurred; error = {:?}", e);
            }
        });
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    db: Arc<Mutex<HashMap<Token, Token>>>,
) -> Result<()> {
    let mut buffer = [0u8; 512];
    loop {
        let res = stream.read(&mut buffer).await?;

        if res == 0 {
            println!("Closing connection");
            break;
        }

        let parser = Parser::new(String::from_utf8(buffer[..res].to_vec()).unwrap());

        for token in parser {
            match token {
                Token::Array(_, tokens) => {
                    println!("Got array: {:?}", tokens);

                    let command = tokens[0].clone();

                    match command {
                        Command(CommandIdent::Ping) => {
                            stream
                                .write(
                                    Token::SimpleString("PONG".to_string())
                                        .to_string()
                                        .as_bytes(),
                                )
                                .await?;
                        }
                        Command(CommandIdent::Echo) => {
                            let message = tokens[1].clone();

                            stream.write(message.to_string().as_bytes()).await?;
                        }
                        Command(CommandIdent::Set) => {
                            let key = tokens[1].clone();
                            let value = tokens[2].clone();

                            let mut db = db.lock().await;

                            db.insert(key.clone(), value.clone());

                            stream
                                .write(Token::SimpleString("OK".to_string()).to_string().as_bytes())
                                .await?;
                        }
                        Command(CommandIdent::Get) => {
                            let key = tokens[1].clone();

                            let db = db.lock().await;

                            let value = db.get(&key).unwrap_or(&Token::NullBulkString);

                            println!("Writing value: {:?}", value.to_string().as_bytes());

                            stream.write(value.to_string().as_bytes()).await?;
                        }
                        _ => {
                            println!("Got token: {:?}", tokens);

                            stream
                                .write(Token::SimpleString("OK".to_string()).to_string().as_bytes())
                                .await?;
                        }
                    }
                }
                _ => {
                    println!("Got token: {:?}", token);
                }
            }
        }

        println!("Sent: +OK");
    }

    Ok(())
}
