use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::{env, fs};

struct Response {
    status: String,
    headers: String,
    body: String,
}

#[derive(Debug)]
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
    } else if binding != "" {
        headers.push(binding.as_str());
    }

    if headers.contains(&header) {
        headers.retain(|&x| x != header);
    }

    headers.push(header);

    response.headers = headers.join("\r\n");
}

fn get_header(request: &Request, header: &str) -> String {
    if request.headers.contains("\r\n") {
        let headers = request.headers.split("\r\n").collect::<Vec<&str>>();
        headers.iter().find(|&x| x.contains(header)).unwrap().split(':').collect::<Vec<&str>>()[1].trim().to_string()
    } else if request.headers.contains(header) {
        request.headers.split(':').collect::<Vec<&str>>()[1].trim().to_string()
    } else {
        "".to_string()
    }
}

fn get_request(mut stream: &TcpStream) -> Request {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request_str = String::from_utf8_lossy(&buffer);
    let request_str = request_str.trim_end_matches('\0');

    eprintln!("request_str: {}", request_str);
    let method = String::from(request_str.lines().next().unwrap().split(' ').collect::<Vec<&str>>()[0]);
    let uri = String::from(request_str.lines().next().unwrap().split(' ').collect::<Vec<&str>>()[1]);
    let version = String::from(request_str.lines().next().unwrap().split(' ').collect::<Vec<&str>>()[2]);
    let headers = String::from(request_str.lines().skip(1).collect::<Vec<&str>>().join("\r\n").trim());
    let body = String::from(request_str.lines().skip(2).collect::<Vec<&str>>().join("").trim());
    
    let request = Request {
        method: String::from(method),
        uri: String::from(uri),
        version: String::from(version),
        headers: String::from(headers),
        body: String::from(body),
    };
    eprintln!("request:\n{:#?}", request);
    request
}

enum Status {
    Ok,
    NotFound,
    Created,
}

impl Status {
    fn to_string(&self) -> String {
        match self {
            Status::Ok => String::from("HTTP/1.1 200 OK"),
            Status::NotFound => String::from("HTTP/1.1 404 Not Found"),
            Status::Created => String::from("HTTP/1.1 201 Created"),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let request = get_request(&stream);

    let mut response = Response {
        status: String::from(""),
        headers: String::from(""),
        body: String::from(""),
    };

    let env_args = env::args().collect::<Vec<String>>();
    
    response.status = match request.uri.as_str() {
        "/" => Status::Ok.to_string(),
        "/user-agent" => {
            handle_header(&mut response, "Content-Type: text/plain");
            let user_agent = get_header(&request, "User-Agent");
            response.body = String::from(user_agent.trim());
            let len = response.body.len();
            handle_header(&mut response, format!("Content-Length: {}", len).as_str());
            Status::Ok.to_string()
        },
        // Route: /echo/{str}
        echo_str if echo_str.starts_with("/echo/") => {
            let echo_str = echo_str.split('/').collect::<Vec<&str>>()[2];
            response.body = String::from(echo_str);
            handle_header(&mut response, "Content-Type: text/plain");
            let len = response.body.len();
            handle_header(&mut response, format!("Content-Length: {}", len).as_str());
            Status::Ok.to_string()
        },
        // Route: /files/{filename}
        filename if filename.starts_with("/files/") => {
            let filename = filename.replace("/files/", "");
            let dir = env_args.iter().position(|x| x == "--directory")
                              .map(|x| env_args[x + 1].clone())
                              .unwrap_or_default();

            if dir.is_empty() {
                Status::NotFound.to_string()
            };

            match fs::read_to_string(format!("{}/{}", dir, filename)) {
                Ok(file_contents) => {
                    response.body = file_contents;
                    handle_header(&mut response, "Content-Type: text/plain");
                    handle_header(&mut response, &format!("Content-Length: {}", response.body.len()));
                    Status::Ok.to_string()
                },
                Err(_) => Status::NotFound.to_string(),
            }
        },
        _ => Status::NotFound.to_string(),
    };

    let response_str = format!("{}\r\n{}\r\n\r\n{}", response.status, response.headers, response.body);
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
