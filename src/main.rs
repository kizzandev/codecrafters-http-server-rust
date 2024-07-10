use std::net::TcpListener;
use std::io::Write;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(&mut _stream) => {
                eprintln!("accepted new connection");

                _stream.write(b"HTTP/1.1 200 OK\r\n\r\n");
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}
