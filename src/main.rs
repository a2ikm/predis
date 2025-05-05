use std::io::prelude::*;
use std::net::{TcpStream, TcpListener};

mod resp;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        handle_client(stream?)?;
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()>{
    let mut request_buf = [0; 4096];
    let _size = stream.read(&mut request_buf).unwrap();

    match resp::decode(&request_buf) {
        Ok(value) => {
            println!("decode success: {:?}", value);
        },
        Err(e) => {
            println!("decode error: {:?}", e);
        }
    }

    stream.write_all(&request_buf)?;
    Ok(())
}
