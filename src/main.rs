use std::net::{TcpListener, TcpStream};
use std::io::{Write, BufRead, BufReader, Read};
use std::collections::HashMap;

use std::thread;
use std::fs;
use std::env;

use itertools::Itertools;

enum HTTPMethod {
    GET, 
    POST, 
    PUT, 
    DELETE
}

enum ContentType {
    PLAIN,
    JSON,
    OCTET,
}

impl HTTPMethod {
    fn as_str(&self) -> &str {
        match self {
            HTTPMethod::GET => "GET",
            HTTPMethod::POST => "POST",
            HTTPMethod::PUT => "PUT",
            HTTPMethod::DELETE => "DELETE",
        }
    }

    fn from_str(method: &str) -> Option<Self> {
        match method {
            "GET" => Some(HTTPMethod::GET), 
            "PUT" => Some(HTTPMethod::PUT), 
            "POST" => Some(HTTPMethod::POST),
            "DELETE" => Some(HTTPMethod::DELETE),
            _ => None, 
        }
    }
}

impl ContentType {
    fn as_str(&self) -> &str {
        match self {
            ContentType::PLAIN => "text/plain",
            ContentType::JSON => "application/json",
            ContentType::OCTET => "application/octet-stream",
        }
    }

    fn from_str(content_type: &str) -> Option<ContentType> {
        match content_type {
            "text/plain" => Some(ContentType::PLAIN),
            "application/json" => Some(ContentType::JSON),
            "application/octet-stream" => Some(ContentType::OCTET),
            _ => None
        }
    }
}


struct HTTPResponse {
    version: String,
    status_code: u16,
    status_msg: String, 
    headers: HashMap<String, String>,
    body: String,
    content_type: ContentType
}

impl HTTPResponse {
    fn new(version: String, status_code: u16, status_msg: String) -> Self {
        HTTPResponse {
            version,
            status_code,
            status_msg,
            headers: HashMap::new(),
            body: String::new(),
            content_type: ContentType::PLAIN,
        }
    }

    fn to_string(self) -> String {
        let mut response = format!(
            "{} {} {}\r\n",
            self.version,
            self.status_code,
            self.status_msg
        );

        for (key, value) in self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }

        response.push_str("\r\n");
        response.push_str(&self.body);

        response
    }
}

struct HTTPRequest {
    method: HTTPMethod,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HTTPRequest {
    fn new() -> Self {
        HTTPRequest {
            method: HTTPMethod::GET,
            path: String::new(),
            version: String::new(),
            headers: HashMap::new(),
            body: None,
        }
    }

    fn parse_request(&mut self, stream: &mut TcpStream) -> Result<(), String> {
        let mut reader = BufReader::new(stream);

        // Read the request line
        let mut req_line = String::new();
        if let Err(e) = reader.read_line(&mut req_line) {
            return Err(format!("Error reading request line: {}", e));
        }

        let parts: Vec<&str> = req_line.trim().split_whitespace().collect();
        if parts.len() != 3 {
            return Err("Invalid request line".to_string());
        }
        if let Some(method) = HTTPMethod::from_str(parts[0]) {
            self.method = method;
        }
        else {
            return Err(format!("Invalid HTTP method: {}", parts[0]));
        }
        //These 2 parameters could use some rigorous checking
        self.path = parts[1].to_string();
        self.version = parts[2].to_string();

        // Read the headers into a vector
        let mut headers = Vec::new();
        for line in reader.by_ref().lines() {
            let line = match line {
                Ok(line) => line,
                Err(e) => return Err(format!("Error reading header line: {}", e)),
            };

            if line.is_empty() {
                break;
            }
            headers.push(line);
        }

        // Process headers
        for line in headers {
            let mut header_parts = line.splitn(2, ':');
            if let (Some(key), Some(value)) = (header_parts.next(), header_parts.next()) {
                self.headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        // Read the optional body
        if let Some(_length_str) = self.headers.get("Content-Length") {
            /*
            let length: usize = match length_str.parse() {
                Ok(len) => len,
                Err(e) => return Err(format!("Error parsing Content-Length: {}", e)),
            };

            let mut body = vec![0; length];
            if let Err(e) = reader.read_exact(&mut body) {
                return Err(format!("Error reading body: {}", e));
            }

            self.body = match String::from_utf8(body) {
                Ok(body_str) => Some(body_str),
                Err(e) => return Err(format!("Error parsing body as UTF-8: {}", e)),
            };
            */
            self.body = None;
        } else {
            self.body = None;
        }

        Ok(())
    }
}

// This function encapsulates the behaviour of the HTTP server
fn http_server_response(request: &HTTPRequest) -> HTTPResponse {
    // By default this will be the default response
    let path = request.path.as_str();
    let is_echo_endpoint: bool = path.starts_with("/echo");
    let is_agent_endpoint: bool = path.starts_with("/user-agent");
    let is_file_endpoint: bool = path.starts_with("/files");

    //Give default values to endpoints
    let (status_code, status_msg) = match path { 
        "/" => (200, "OK"),
        _ if is_echo_endpoint => (200, "OK"),
        _ if is_agent_endpoint => (200, "OK"),
        _ if is_file_endpoint => (200, "OK"),
        _ => (404, "Not Found"),
    };

    let mut response = HTTPResponse::new(
        request.version.clone(),
        status_code,
        status_msg.to_string(),
    );

    if is_echo_endpoint {
        response.content_type = ContentType::PLAIN;
        response.headers.insert(
            "Content-Type".to_string(),
             response.content_type.as_str().to_string()
        );
        response.body.push_str(path.trim_start_matches("/echo/"));
    } 
    else if is_agent_endpoint {
        response.content_type = ContentType::PLAIN;
        response.headers.insert(
            "Content-Type".to_string(),
            response.content_type.as_str().to_string()
        );

        if let Some(user_agent) = request.headers.get("User-Agent") {
            response.body.push_str(user_agent);
        } else {
            response.status_code = 404;
            response.status_msg = "User-Agent header not found".to_string();
        }
    }
    else if is_file_endpoint {
        // Extract the file name from the path
        let file_name = path.trim_start_matches("/files/");
    
        // Collect command line arguments and build the absolute path to the file
        let env_args: Vec<String> = env::args().collect();
        let directory = env_args.get(2).unwrap_or(&String::from(".")).clone();
        let file_path = format!("{}/{}", directory, file_name);
        let file_result = fs::read(&file_path);
    
        let response_body = match file_result {
            Ok(file_content) => match String::from_utf8(file_content) {
                Ok(content) => {
                    response.content_type = ContentType::OCTET;
                    response.headers.insert(
                        "Content-Type".to_string(),
                        response.content_type.as_str().to_string(),
                    );
                    content
                }
                Err(e) => {
                    eprintln!("Error converting file content to String: {}", e);
                    response.status_code = 500;
                    response.status_msg = "Internal Server Error".to_string();
                    //Graceful error message
                    //"Error: Could not read file content as UTF-8".to_string()
                    "".to_string()
                }
            },
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                response.status_code = 404;
                response.status_msg = "Not Found".to_string();
                //Graceful error message
                //"Error: File not found".to_string()
                "".to_string()
            }
        };
    
        response.body = response_body;
    }

    // Set Content-Length header
    let content_length = response.body.len();
    response.headers.insert(
        "Content-Length".to_string(),
        content_length.to_string()
    );
    response
}


fn handle_tcp_stream_connect(tcp_stream: &mut TcpStream) -> Result<(), std::io::Error>{
    println!(
        "Accepted new connection from TCP connection with socket address {}",
         tcp_stream.peer_addr()?
    );

    let mut request: HTTPRequest = HTTPRequest::new();

    // Parse the incoming HTTP request
    if let Err(e) = request.parse_request(tcp_stream) {
        eprintln!("Error parsing request: {}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
    }

    // Let the server elaborate a response for the request
    let response = http_server_response(&request);
    let response_string = response.to_string();

    // Write the response to the TCP connection (Stream)
    if let Err(e) = tcp_stream.write_all(response_string.as_bytes()) {
        eprintln!("Error sending the response: {}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, e));
    }

    Ok(())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // Multithreaded solution 
                thread::spawn(move || {
                    if let Err(e) = handle_tcp_stream_connect(&mut stream) {
                        eprintln!("Error handling connection: {}", e);
                    }
                });

            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
