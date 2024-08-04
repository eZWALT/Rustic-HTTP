use std::net::{TcpListener, TcpStream};
use std::io::{Write, BufRead, BufReader, Read};
use std::collections::HashMap;

struct HTTPRequest {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HTTPRequest {
    fn new() -> Self {
        HTTPRequest {
            method: String::new(),
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
        self.method = parts[0].to_string();
        self.path = parts[1].to_string();
        //println!("The path of this request is: {}", self.path);
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
        if let Some(length_str) = self.headers.get("Content-Length") {
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
        } else {
            self.body = None;
        }

        Ok(())
    }
}

fn write_response(request: &HTTPRequest, stream: &mut TcpStream) -> Result<(), std::io::Error> {
    //By default this will be the default response
    let (status_code, status_msg) = match request.path.as_str() { 
        "/" => (200, "OK"),
        _ => (404, "Not Found"),
    };

    let response = format!(
        "{} {} {}\r\n\r\n",
        request.version,
        status_code, 
        status_msg,
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Accepted new connection");
                let mut request = HTTPRequest::new();

                // Parse the incoming HTTP request
                if let Err(e) = request.parse_request(&mut stream) {
                    eprintln!("Error parsing request: {}", e);
                    continue; // Skip to the next connection
                }

                // Elaborate an write a response for the request
                if let Err(e) = write_response(&request, &mut stream) {
                    eprintln!("Error writing the response: {}", e);
                    continue;
                }
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
