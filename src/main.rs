use log::{error, info};
use redis_rust::event_loop::{Command, EventLoop};
use redis_rust::traits::{DefaultLogger, RedisLogger};
use redis_rust::types::{RedisError, RedisOp, RedisRequest, RedisType, Result};
use redis_rust::Redis;
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

pub const SERVER_ADDRESS: &str = "127.0.0.1:6379";
pub const THREAD_COUNT: usize = 4;

pub fn handle_connection<T: RedisLogger + Clone + Default>(
    mut stream: TcpStream,
    redis: Arc<Mutex<Redis<T>>>,
) -> Result<()> {
    loop {
        let mut buf_reader = BufReader::new(stream.try_clone().map_err(|err| {
            RedisError::from(format!("Error while cloning stream: {}", err.to_string()))
        })?);

        let mut buf = [0u8; 256];
        let res = buf_reader.read(&mut buf);

        if res.is_err() {
            return Err(RedisError::from(res.unwrap_err().to_string()));
        }

        let buf = String::from_utf8(buf.to_vec()).map_err(|err| {
            RedisError::from(format!("Error while parsing buffer: {}", err.to_string()))
        })?;

        let req = RedisRequest::parse(buf).unwrap_or_else(|err| {
            RedisRequest::new(
                RedisOp::Error,
                Some(RedisType::String(err.to_string())),
                None,
            )
        });

        let result = redis
            .lock()
            .map_err(|err| RedisError::from(err.to_string()))?
            .handle_command(req);

        stream
            .write(result.as_string_formatted().as_bytes())
            .map_err(|err| {
                RedisError::from(format!(
                    "Error while writing to stream: {}",
                    err.to_string()
                ))
            })?;
    }
}

fn main() {
    simple_logger::init_with_env().unwrap();

    let mut e_loop = EventLoop::new(THREAD_COUNT);
    let redis = Arc::new(Mutex::new(Redis::<DefaultLogger>::with_capacity(100)));
    let listener = TcpListener::bind(SERVER_ADDRESS).unwrap_or_else(|err| {
        panic!(
            "Error while binding to address {}: {}",
            SERVER_ADDRESS,
            err.to_string()
        )
    });

    info!("Server is listening on: {}", SERVER_ADDRESS);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let redis_clone = Arc::clone(&redis);
                e_loop
                    .send(Command::Task(Box::new(move || {
                        handle_connection(stream, redis_clone)
                            .unwrap_or_else(|err| error!("got error: {:?}", err));
                    })))
                    .unwrap_or_else(|err| error!("got error: {:?}", err));
            }
            Err(err) => error!("got error: {:?}", err),
        }
    }

    e_loop
        .terminate()
        .unwrap_or_else(|err| error!("Error while terminating event loop: {}", err.to_string()));
}
