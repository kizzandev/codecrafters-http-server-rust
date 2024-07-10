use std::net::TcpListener;
use std::io::Write;

struct Response {
    status: String,
    headers: String,
    body: String,
}

struct Request {
    method: String,
    uri: String,
    version: String,
    headers: String,
    body: String,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                eprintln!("accepted new connection");

                // _stream.write(b"HTTP/1.1 200 OK\r\n\r\n");

                // Extract METHOD, URI, HTTP version, headers, and body
                let mut request = Request {
                    method: String::from(""),
                    uri: String::from(""),
                    version: String::from(""),
                    headers: String::from(""),
                    body: String::from(""),
                };

                // Extract method from _stream
                eprintln!("The _stream is:\n{:?}", _stream);


                // let status = match request.uri.as_str() {
                //     Some(uri) => {
                //         "HTTP/1.1 200 OK\r\n\r\n"
                //     },
                //     None => {
                //         "HTTP/1.1 404 Not Found\r\n\r\n"
                //     }
                // }
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}
