use std::net::TcpListener;
use std::net::TcpStream;
use std::io::Write;
use std::io::Read;

// struct Response {
//     status: String,
//     headers: String,
//     body: String,
// }

struct Request {
    method: String,
    uri: String,
    version: String,
    headers: String,
    body: String,
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request_str = String::from_utf8_lossy(&buffer);
    eprintln!("request: {}", request_str);

    // The string must start with a method (e.g. GET)
    let method = request_str.lines().next().unwrap().split(' ').collect::<Vec<&str>>()[0];
    let uri = request_str.lines().next().unwrap().split(' ').collect::<Vec<&str>>()[1];
    let version = request_str.lines().next().unwrap().split(' ').collect::<Vec<&str>>()[2];
    let headers = request_str.lines().skip(1).collect::<Vec<&str>>().join("\n");
    let body = request_str.lines().skip(2).collect::<Vec<&str>>().join("\n");
    
    let request = Request {
        method: String::from(method),
        uri: String::from(uri),
        version: String::from(version),
        headers: String::from(headers),
        body: String::from(body),
    };

    eprintln!("method: {}", request.method);
    eprintln!("uri: {}", request.uri);
    eprintln!("version: {}", request.version);
    eprintln!("headers: {}", request.headers);
    eprintln!("body: {}", request.body);
    
    // let status = match request.uri.as_str() {
    //     Some(uri) => {
    //         "HTTP/1.1 200 OK\r\n\r\n"
    //     },
    //     None => {
    //         "HTTP/1.1 404 Not Found\r\n\r\n"
    //     }
    // }
    let _ = stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n");
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
