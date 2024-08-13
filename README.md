# Rustic-HTTP 🦀⚡🌐

## Description

Welcome to **Rustic-HTTP**, a lightweight and robust HTTP server written in Rust. This project is designed to be simple, fast, and for educative purposes, making it perfect for anyone looking to build a custom HTTP server with minimal overhead.


### Endpoints

- **/**: Root endpoint.
- **/echo/{message}**: Echoes back the message provided in the URL.
- **/user-agent**: Returns the User-Agent header from the request.
- **/files/{filename}**: Statically retrieve (GET) or upload (POST) files to a shared folder.

### Features

- Multithreaded
- Multiple endpoints
- Support for GET/POST methods
- Static file serving
- GZIP Compression

## Usage 

To manually build this HTTP server you will need Rust and Cargo (>= 1.75): 

```sh
git clone https://github.com/eZWALT/Rustic-HTTP.git

cd Rustic-HTTP

./compilation_script.sh --directory=$(pwd)

```

## Testing 

For easy testing a tool like cURL can be useful. Some examples are shown below:

```sh
curl -v http://localhost:4221/echo/sus

curl -v -H "Accept-Encoding: gzip" http://localhost:4221/user-agent

```

## Configuration

- Port: Modify the bind address in the main.rs file to change the server's port.
- Root Directory: Specify the directory to serve files from as an argument when running the server.

## To-Do

- [ ] Implementation of query string parameters parsing
- [ ] Create a config file for ips/ports/paths... values
- [ ] Implement some form of authentification
- [ ] Improve /files endpoint 
- [ ] Asynchronous responses 
- [ ] /time endpoint
- [ ] /status endpoint
- [ ] /info endpoint
- [ ] /list endpoint
- [ ] Small refactor into multiple files




## License

This project is licensed under the GPLv3 License - see the [LICENSE](./LICENSE) file for details.

