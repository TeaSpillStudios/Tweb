use markdown::file_to_html;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::Path;

const LIVE_MODE: bool = true;
const CSS: &str = "
<style>
    :root {
    	font-family: 'Inter', sans-serif;
    }
    
    @supports (font-variation-settings: normal) {
    	:root {
    		font-family: 'Inter var', sans-serif;
    	}
    }
    
    a {
    	color: white;
    }
    
    h1, h2, h3, h4, h5 {
    	text-decoration: underline;
    }
    
    h1 {
    	text-align: center;
    }
    
    #footer {
    	text-align: center;
    	font-size: 75%;
    }
    
    body {
    	background-color: rgb(24, 26, 27);
    	color: rgb(225, 223, 219);
    	margin-left: 5em;
    	margin-right: 5em;
    }
    
    #line_break {
    	visibility: hidden;
    	margin: 0.5em;
    }
    
    footer {
        position: absolute;
        left: 0;
        bottom: 0;
        height: 50px;
        width: 100%;
        overflow: hidden;
    }
</style>
";

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
        file_to_html(Path::new("index.md")).unwrap()
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
    let data = format!("<!DOCTYPE html>{}{}", CSS, markdown_loader.load());
    let length = data.len();

    let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{data}");

    stream
        .write_all(response.as_bytes())
        .expect("Failed to write to stream TCP.");
}
