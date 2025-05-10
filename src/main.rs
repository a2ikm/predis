use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

mod resp;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                println!("connection error: {:?}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut request_buf = [0; 4096];
    let _size = stream.read(&mut request_buf).unwrap();

    let response_buf = handle_request(&request_buf);

    stream
        .write_all(&response_buf)
        .expect("response_buf should be written");
}

fn handle_request(request_buf: &[u8]) -> Vec<u8> {
    let request = match resp::decode(request_buf) {
        Some(value) => value,
        None => return resp::encode(&resp::Value::SimpleString("NG".as_bytes().to_vec())),
    };

    resp::encode(&request)
}
