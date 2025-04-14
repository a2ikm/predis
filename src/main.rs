use std::io::prelude::*;
use std::net::{TcpStream, TcpListener};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8765")?;

    for stream in listener.incoming() {
        handle_client(stream?)?;
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()>{
    let mut request_buf = [0; 4096];
    let _size = stream.read(&mut request_buf).unwrap();

    stream.write_all(&request_buf)?;
    Ok(())
}
