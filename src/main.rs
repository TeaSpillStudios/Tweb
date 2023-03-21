use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7250").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_request(stream);
    }
}

fn handle_request(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");

    stream
        .write_all(b"HTTP/1.1 200 OK\r\n\r\n")
        .expect("Failed to write to stream TCP.");
}
