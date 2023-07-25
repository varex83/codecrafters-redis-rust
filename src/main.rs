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

        stream.write(b"+PONG\r\n").await?;

        println!("Sent: +OK");
    }

    Ok(())
}
