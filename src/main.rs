use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use bytes::{BufMut, BytesMut};
use redis_starter_rust::{command::Command, resp};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let shared_storage = Arc::new(Mutex::new(HashMap::new()));

    let server = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Server listening on {}", server.local_addr()?);

    loop {
        match server.accept().await {
            Ok((client, _)) => {
                println!("Accepted new connection");
                let storage_ref = Arc::clone(&shared_storage);

                tokio::spawn(async move {
                    if let Err(e) = process_client(client, storage_ref).await {
                        eprintln!("Error processing client: {}", e);
                    }
                });
            }
            Err(err) => eprintln!("Connection error: {}", err),
        }
    }
}

async fn process_client(
    mut client: TcpStream,
    storage: Arc<Mutex<HashMap<String, String>>>,
) -> Result<()> {
    let mut buffer = BytesMut::with_capacity(512);
    loop {
        let read_bytes = client.read_buf(&mut buffer).await?;
        if read_bytes == 0 {
            println!("Client disconnected");
            break;
        }

        if let Some((_, parsed_resp)) = resp::RespValue::from_bytes(&buffer)? {
            if let Some(cmd) = Command::from_resp_value(&parsed_resp) {
                match cmd {
                    Command::Ping(message) => {
                        let mut response = BytesMut::with_capacity(512);
                        response.put_slice(b"+");
                        response.put_slice(message.as_bytes());
                        response.put_slice(b"\r\n");

                        client.write_all(&response).await?;
                    }
                    Command::Echo(message) => {
                        let mut response = BytesMut::with_capacity(512);
                        response.put_slice(b"+");
                        response.put_slice(message.as_bytes());
                        response.put_slice(b"\r\n");

                        client.write_all(&response).await?;
                    }
                    Command::Get(key) => {
                        let mut response = BytesMut::with_capacity(512);
                        let locked_storage = storage.lock().await;
                        if let Some(value) = locked_storage.get(&key) {
                            response.put_slice(b"+");
                            response.put_slice(value.as_bytes());
                            response.put_slice(b"\r\n");
                        } else {
                            response.put_slice(b"-ERR Key not found\r\n");
                        }

                        client.write_all(&response).await?;
                    }
                    Command::Set(key, value) => {
                        let mut locked_storage = storage.lock().await;
                        locked_storage.insert(key, value);
                        client.write_all(b"+OK\r\n").await?;
                    }
                    _ => client.write_all(b"-ERR Unsupported command\r\n").await?,
                }
            } else {
                client.write_all(b"-ERR Invalid command\r\n").await?;
            }
            buffer.clear();
        }
    }
    Ok(())
}
