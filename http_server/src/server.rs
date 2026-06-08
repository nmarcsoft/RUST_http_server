use std::{io::Read, net::{TcpListener, TcpStream}, io::Write};

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
enum ResponseType {
    ApplicationJson,
}

#[derive(Debug)]
struct HttpResponse {
    request_type : RequestType,
    reponse_request_type: ResponseType,
    response_content_type: String,
    response_content_length: u32,

    response_body: String,
}

impl HttpResponse {
    fn new(request_type : RequestType, response_type : ResponseType, body : Option<String>) -> Option<HttpResponse> {

        if request_type == RequestType::Get {
            let body_unwrapped : String = body.unwrap_or(String::from("{message: Hello World}"));

            Some(HttpResponse {
                request_type : RequestType::Get,
                reponse_request_type : ResponseType::ApplicationJson,

                response_content_type : String::from("application/json"),
                response_content_length : body_unwrapped.len() as u32,

                response_body : body_unwrapped,
            })
        } else {
            None
        }
    }

    fn send_response(self, stream : &mut TcpStream) -> std::io::Result<()> {

        let mut to_write : String = String::new();
        let intro : &str = "HTTP/1.1 200 OK\r\n";
        to_write.push_str(intro);

        let content_type_str : String = format!("Content-Type: {}\r\n", self.response_content_type);
        to_write.push_str(content_type_str.as_str());

        let content_length_str : String = format!("Content-Length: {}\r\n", self.response_content_length);
        to_write.push_str(content_length_str.as_str());

        to_write.push_str("\r\n");
        to_write.push_str(self.response_body.as_str());

        let result = stream.write_all(to_write.as_bytes());
        result
    }
}

#[derive(Debug)]
struct HttpReader {
    http_request_type: Option<RequestType>,
    http_host: String,
    http_user_agent: String,
}

fn get_request_type(line: &str, http_request : &mut HttpReader) {
    let first_word = line.split(" ").nth(0);

    http_request.http_request_type = match first_word {
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

fn get_host_ip(line : &str, http_request : &mut HttpReader) {

    let parser : Vec<&str> = line.splitn(2, ": ").collect();

    if parser.len() != 2 {
        http_request.http_host = String::from("Error");
    } else {
        http_request.http_host = String::from(parser[1]);
    }
}

fn get_user_agent(line : &str, http_request : &mut HttpReader) {

    let parser : Vec<&str> = line.splitn(2, ": ").collect();

    if parser.len() != 2 {
        http_request.http_user_agent = String::from("Error");
    } else {
        http_request.http_user_agent = String::from(parser[1]);
    }
}

fn handle_client(mut stream : TcpStream){
    println!("Recived Tcp request");

    let mut read_buffer = [0; 2048];
    let result = stream.read(&mut read_buffer[..]);

    let n =
        match result {
            Ok(n) => n,
            Err(_) => return,
        };

    let usable_buffer = &read_buffer[0..n];
    let result_conversion = str::from_utf8(usable_buffer);

    let converted =
        match result_conversion {
            Ok(str) => str,
            Err(v) => return,
        };

    let mut part = converted.split("\r\n");


    let mut http_request : HttpReader = HttpReader {
        http_request_type :  None,
        http_host : String::new(),
        http_user_agent :  String::new(),
    };

    // Parsing ligne à ligne pour créer la structure
    if let Some(line) = part.nth(0) {
        get_request_type(line, &mut http_request);
    }

    for line in part {
        if line.starts_with("Host") {
            get_host_ip(line, &mut http_request);
        } else if line.starts_with("User-Agent") {
            get_user_agent(line, &mut http_request);
        }
    }
    println!("{:#?}", http_request);

    let http_response : Option<HttpResponse> = HttpResponse::new(http_request.http_request_type.unwrap(), ResponseType::ApplicationJson, None);

    if http_response.is_some() {
        let result = http_response.unwrap().send_response(&mut stream);
    }
}

pub fn init_tcp_listener() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        handle_client(stream?);
    }
    Ok(())
}
