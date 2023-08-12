mod parser;

use crate::parser::Token::Command;
use crate::parser::{CommandIdent, Parser, Token};
use anyhow::Result;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub struct DbValue {
    value: Token,
    ttl: Option<u128>,
}

pub struct Db {
    db: HashMap<Token, DbValue>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            db: HashMap::<Token, DbValue>::new(),
        }
    }

    pub fn get(&self, key: &Token) -> Token {
        let value = self.db.get(key);

        match value {
            Some(value) => {
                let ttl = value.ttl;

                if ttl.is_none() {
                    return value.value.clone();
                }

                let ttl = ttl.unwrap();

                let time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();

                if ttl > time {
                    value.value.clone()
                } else {
                    Token::NullBulkString
                }
            }
            None => Token::NullBulkString,
        }
    }

    pub fn set(&mut self, key: Token, value: DbValue) {
        self.db.insert(key, value);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db = Db::new();
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

async fn handle_connection(mut stream: TcpStream, db: Arc<Mutex<Db>>) -> Result<()> {
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

                            let expiry = match tokens.get(3) {
                                Some(Command(CommandIdent::Px)) => {
                                    let duration = tokens[4].clone();

                                    let now = SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis();

                                    match duration {
                                        Token::Integer(duration) => Some(now + (duration as u128)),
                                        Token::BulkString(_, duration) => {
                                            Some(now + u128::from_str(duration.as_str()).unwrap())
                                        }
                                        _ => None,
                                    }
                                }
                                _ => None,
                            };

                            println!("Setting key: {:?} with value: {:?}", key, value);
                            println!("Expiry: {:?}", expiry);

                            let mut db = db.lock().await;

                            db.set(
                                key.clone(),
                                DbValue {
                                    value: value.clone(),
                                    ttl: expiry,
                                },
                            );

                            stream
                                .write(Token::SimpleString("OK".to_string()).to_string().as_bytes())
                                .await?;
                        }
                        Command(CommandIdent::Get) => {
                            let key = tokens[1].clone();

                            let db = db.lock().await;

                            let value = db.get(&key);

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
