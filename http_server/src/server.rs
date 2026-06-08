use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, PartialEq)]
enum RequestType {
    Get,
    Post,
    Delete,
    Put,
    Patch,
    Head,
    Trace,
    Options,
    Connect,
}

#[derive(Debug)]
struct HttpRequest {
    method: Option<RequestType>,
    target: String,
    host: String,
    user_agent: String,
}

impl HttpRequest {
    fn parse(lines : Vec<&str>) -> Self {

        let request_type : Option<RequestType>;
        let request_target : String;
        let mut host : String = String::new();
        let mut user_agent : String = String::new();

        request_type = parse_request_line(lines[0]);
        request_target = String::from(lines[0].split(" ").nth(1).unwrap());
        for line in lines {
            if let Some((key, value)) = line.split_once(": ") {
                match key {
                    "Host" => host = value.to_string(),
                    "User-Agent" => user_agent = value.to_string(),
                    _ => {}
                }
            }
        };

        HttpRequest { method: request_type, target: request_target, host, user_agent }
    }
}

#[derive(Debug)]
struct HttpResponse {
    status_code: u16,
    content_type: String,
    body: String,
}

impl HttpResponse {
    fn ok(content_type: &str, body: String) -> Self {
        HttpResponse {
            status_code: 200,
            content_type: content_type.to_string(),
            body,
        }
    }

    fn not_found() -> Self {
        HttpResponse {
            status_code: 404,
            content_type: String::from("application/json"),
            body: String::from("{\"error\": \"Not Found\"}"),
        }
    }

    fn send(self, stream: &mut TcpStream) -> std::io::Result<()> {
        let status_text = match self.status_code {
            200 => "OK",
            404 => "Not Found",
            _ => "Internal Server Error",
        };

        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            self.status_code,
            status_text,
            self.content_type,
            self.body.len(),
            self.body,
        );

        stream.write_all(response.as_bytes())
    }
}


fn parse_request_line(line: &str) -> Option<RequestType> {
    let mut parts = line.split(' ');

    match parts.next() {
        Some("GET") => Some(RequestType::Get),
        Some("POST") => Some(RequestType::Post),
        Some("DELETE") => Some(RequestType::Delete),
        Some("PUT") => Some(RequestType::Put),
        Some("PATCH") => Some(RequestType::Patch),
        Some("HEAD") => Some(RequestType::Head),
        Some("TRACE") => Some(RequestType::Trace),
        Some("OPTIONS") => Some(RequestType::Options),
        Some("CONNECT") => Some(RequestType::Connect),
        _ => None,
    }
}

fn route(request: HttpRequest) -> HttpResponse {
    match request.method {
        Some(RequestType::Get) => handle_get(request.target),
        _ => HttpResponse::not_found(),
    }
}

fn handle_get(target: String) -> HttpResponse {
    if target == "/" {
        HttpResponse::ok("application/json", String::from("{\"message\": \"Hello World\"}"))
    } else if let Some(book_name) = target.strip_prefix("/book/") {
        if book_name.is_empty() {
            return HttpResponse::not_found();
        }
        HttpResponse::ok(
            "application/json",
            format!("{{\"message\": \"On cherche le livre {book_name}\"}}"),
        )
    } else {
        HttpResponse::not_found()
    }
}

fn handle_client(mut stream: TcpStream) {
    println!("Received TCP request");

    let mut read_buffer = [0; 2048];
    let n = match stream.read(&mut read_buffer) {
        Ok(n) => n,
        Err(_) => return,
    };

    let converted = match str::from_utf8(&read_buffer[..n]) {
        Ok(s) => s,
        Err(_) => return,
    };
    let lines : Vec<&str>= converted.split("\r\n").collect();
    let request = HttpRequest::parse(lines);
    println!("{:#?}", request);
    let _ = route(request).send(&mut stream);
}

pub fn init_tcp_listener() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    for stream in listener.incoming() {
        handle_client(stream?);
    }
    Ok(())
}
