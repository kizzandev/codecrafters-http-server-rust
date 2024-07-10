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

fn handle_header(mut response: &mut Response, header: &str) {
    let mut headers = response.headers.clone().split("\r\n").collect::<Vec<&str>>();

    if !headers.contains(&header) {
        headers.push(header);
    } else {
        headers.retain(|&x| x != header);
    }

    response.headers = headers.join("\r\n");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request_str = String::from_utf8_lossy(&buffer);
    eprintln!("request: {}", request_str);

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
    
    let status = match request.uri.as_str() {
        "/" => ok,
        // Route: /echo/{str}
        echo_str if echo_str.starts_with("/echo/") => {
            let echo_str = echo_str.split('/').collect::<Vec<&str>>()[2];
            eprintln!("echo_str: {}", echo_str);
            response.body = String::from(echo_str);
            handle_header(&mut response, "Content-Type: text/plain");
            let len = response.body.len();
            handle_header(&mut response, "Content-Length: ".to_owned() + &len.to_string());
            ok
        }
        _ => not_found
    };

    response.status = String::from(status);
    response.headers = String::from(request.headers);
    response.body = String::from("");

    let response_str = format!("{}\r\n{}\r\n{}\r\n\r\n", response.status, response.headers, response.body);
    stream.write(response_str.as_bytes()).unwrap();
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
