use std::net::TcpListener;
use std::net::TcpStream;
use std::io::Write;
use std::io::Read;

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

fn handle_header(response: &mut Response, header: &str) {
    let binding = response.headers.clone();
    let mut headers = Vec::new();
    if binding.contains("\r\n") {
        headers = binding.split("\r\n").collect::<Vec<&str>>();
    } else {
        headers.push(binding.as_str());
    }

    if headers.contains(&header) {
        headers.retain(|&x| x != header);
    }

    headers.push(header);

    response.headers = headers.join("\r\n");
    // eprintln!("headers: {}", response.headers);
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request_str = String::from_utf8_lossy(&buffer);
    // eprintln!("request: {}", request_str);

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

    let mut response = Response {
        status: String::from(""),
        headers: String::from(""),
        body: String::from(""),
    };

    let ok = String::from("HTTP/1.1 200 OK");
    let not_found = String::from("HTTP/1.1 404 Not Found");
    
    response.status = match request.uri.as_str() {
        "/" => ok,
        // Route: /echo/{str}
        echo_str if echo_str.starts_with("/echo/") => {
            let echo_str = echo_str.split('/').collect::<Vec<&str>>()[2];
            response.body = String::from(echo_str);
            handle_header(&mut response, "Content-Type: text/plain");
            let len = response.body.len();
            handle_header(&mut response, format!("Content-Length: {}", len).as_str());
            ok
        }
        _ => not_found
    };

    let mut response_str = String::from(format!("{}\r\n\r\n{}", response.status, response.body));
    if response.headers != "" {
        response_str = format!("{}{}\r\n\r\n{}", response.status, response.headers, response.body);
    }

    eprintln!("response:\n{}", response_str);
    let _ = stream.write(response_str.as_bytes());
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
