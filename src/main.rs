use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:5050").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Connection established");
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let content_type = r#"Content-Type: application/json"#;

    let (status_line, content) = if request_line == "GET / HTTP/1.1" {
        let status_line = "HTTP/1.1 200 OK";
        let content = r#"{"hello": "world"}"#;
        (status_line, content)
    } else {
        let status_line = "HTTP/1.1 404 NOT FOUND";
        let content = r#"{ "message": "not found" }"#;
        (status_line, content)
    };

    let content_len = content.len();
    let response = format!(
        "{status_line}\r\nContent-Length: {content_len}\r\n{content_type}\r\n\r\n{content}"
    );
    stream.write_all(response.as_bytes()).unwrap();
}
