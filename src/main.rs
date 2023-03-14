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

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);

    let content = r#"{"hello": "world"}"#;
    let content_len = content.len();
    let content_type = r#"Content-Type: application/json"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {content_len}\r\n{content_type}\r\n\r\n{content}"
    );
    stream.write_all(response.as_bytes()).unwrap();
}
