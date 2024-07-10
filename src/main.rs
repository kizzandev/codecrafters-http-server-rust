use std::net::TcpListener;
use std::net::TcpStream;
use std::io::Write;
use std::io::Read;

// struct Response {
//     status: String,
//     headers: String,
//     body: String,
// }

// struct Request {
//     method: String,
//     uri: String,
//     version: String,
//     headers: String,
//     body: String,
// }

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    eprintln!("request: {}", String::from_utf8_lossy(&buffer));

    let _ = _stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n");

    
    // let mut request = Request {
    //     method: String::from(""),
    //     uri: String::from(""),
    //     version: String::from(""),
    //     headers: String::from(""),
    //     body: String::from(""),
    // };

    // let status = match request.uri.as_str() {
    //     Some(uri) => {
    //         "HTTP/1.1 200 OK\r\n\r\n"
    //     },
    //     None => {
    //         "HTTP/1.1 404 Not Found\r\n\r\n"
    //     }
    // }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                eprintln!("accepted new connection");
                handle_connection(_stream);
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}
