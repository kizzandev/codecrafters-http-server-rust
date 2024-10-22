use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::{env, fs};
use flate2::{Compression, write::GzEncoder};

#[derive(Debug)]
struct Response {
    status: String,
    headers: String,
    body: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Request {
    method: String,
    uri: String,
    version: String,
    headers: String,
    body: String,
}

fn handle_header(response: &mut Response, header: &str) {
    let header_key = header.split(':').next().unwrap().trim();
    let headers = response.headers.split("\r\n")
                                    .filter(|x| !x.starts_with(header_key))
                                    .collect::<Vec<&str>>();

    response.headers = headers.join("\r\n");

    if !response.headers.contains(header) {
        if !response.headers.is_empty() {
            response.headers.push_str("\r\n");
        }
        response.headers.push_str(header);
    }
}

fn get_header(request: &Request, header: &str) -> String {
    if request.headers.contains("\r\n") {
        let headers = request.headers.split("\r\n").collect::<Vec<&str>>();
        if let Some(h) = headers.iter().find(|&&x| x.starts_with(header)) {
            h.split(':').nth(1).map_or("".to_string(), |v| v.trim().to_string())
        } else {
            "".to_string()
        }
    } else if request.headers.contains(header) {
        request.headers.split(':').nth(1).map_or("".to_string(), |v| v.trim().to_string())
    } else {
        "".to_string()
    }
}

fn get_request(mut stream: &TcpStream) -> Request {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request_str = String::from_utf8_lossy(&buffer);
    let request_str = request_str.trim_end_matches('\0');

    let mut lines = request_str.lines();

    let (method, uri, version) = if let Some(req_line) = lines.next() {
        let mut parts = req_line.split(' ');
        if let (Some(method), Some(uri), Some(version)) = (parts.next(), parts.next(), parts.next()) {
            (method, uri, version)
        } else {
            ("", "", "")
        }
    } else {
        ("", "", "")
    };

    let (headers, body) = {
        let mut req_parts = request_str.split("\r\n\r\n");
        (req_parts.next().unwrap_or(""), req_parts.next().unwrap_or(""))
    };

    let headers = headers.lines().skip(1).collect::<Vec<&str>>().join("\r\n");

    let request = Request {
        method: method.to_string(),
        uri: uri.to_string(),
        version: version.to_string(),
        headers: headers,
        body: body.to_string(),
    };

    eprintln!("{:#?}", request);
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

fn get_file(mut response: &mut Response, env_args: Vec<String>, filename: &str) -> String {
    let filename = filename.replace("/files/", "");
    let dir = env_args.iter().position(|x| x == "--directory")
                      .map(|x| env_args[x + 1].clone())
                      .unwrap_or_default();

    if dir.is_empty() {
        Status::NotFound.to_string()
    } else {
        match fs::read_to_string(format!("{}/{}", dir, filename)) {
            Ok(file_contents) => {
                response.body = file_contents;
                handle_header(&mut response, "Content-Type: application/octet-stream");
                let len = response.body.len();
                handle_header(&mut response, &format!("Content-Length: {}", len));
                Status::Ok.to_string()
            },
            Err(_) => Status::NotFound.to_string(),
        }    
    }
}

fn post_file(request: &Request, env_args: Vec<String>, filename: &str) -> String {
    let filename = filename.replace("/files/", "");
    let dir = env_args.iter().position(|x| x == "--directory")
                      .map(|x| env_args[x + 1].clone())
                      .unwrap_or_default();
    if dir.is_empty() {
        Status::NotFound.to_string()
    } else {
        match fs::write(format!("{}/{}", dir, filename), &request.body) {
            Ok(_) => Status::Created.to_string(),
            Err(_) => Status::NotFound.to_string(),
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
            if request.method == "GET" {
                get_file(&mut response, env_args, filename)
            } else {
                post_file(&request, env_args, filename)
            }
        },
        _ => Status::NotFound.to_string(),
    };

    let mut is_encoded = false;
    let accept_encoding = get_header(&request, "Accept-Encoding");
    let mut compressed = Vec::new();
    
    if !accept_encoding.is_empty() && accept_encoding.contains("gzip") {
        handle_header(&mut response, "Content-Encoding: gzip");

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(response.body.as_bytes()).expect("Failed to write to encoder");

        compressed = encoder.finish().expect("Failed to finish encoding");

        let len = compressed.len();
        handle_header(&mut response, format!("Content-Length: {}", len).as_str());
        is_encoded = true;
    }

    eprintln!("{:#?}", response);
    let response_str = format!(
        "{}\r\n{}\r\n\r\n",
        response.status, response.headers
    );
    let mut buffer = Vec::new();
    buffer.write_all(response_str.as_bytes()).expect("Failed to write to buffer");
    // let mut response_bytes = response_str.as_bytes().to_vec();
    if is_encoded {
        // response_bytes.extend_from_slice(&compressed);
        buffer.write_all(&compressed).expect("Failed to write to buffer");
    } else {
        // response_bytes.extend_from_slice(response.body.as_bytes());
        buffer.write_all(response.body.as_bytes()).expect("Failed to write to buffer");
    }
    let _ = stream.write_all(&buffer).expect("Failed to write to stream");
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
