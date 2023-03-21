use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

const LIVE_MODE: bool = true;

#[derive(Default)]
struct MarkdownLoader {
    cache: String,
}

impl MarkdownLoader {
    pub fn load(&mut self) -> String {
        if !self.cache.is_empty() && !LIVE_MODE {
            self.cache.clone()
        } else {
            Self::load_md()
        }
    }

    fn load_md() -> String {
        todo!("Load MD and return HTML as a String");
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7250").unwrap();
    let mut markdown_loader = MarkdownLoader::default();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_request(stream, &mut markdown_loader);
    }
}

fn handle_request(mut stream: TcpStream, markdown_loader: &mut MarkdownLoader) {
    let status = "HTTP/1.1 200 OK";
    let data = markdown_loader.load();
    let length = data.len();

    let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{data}");

    stream
        .write_all(response.as_bytes())
        .expect("Failed to write to stream TCP.");
}
