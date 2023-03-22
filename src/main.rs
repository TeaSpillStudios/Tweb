use log::{error, info, LevelFilter};
use markdown::file_to_html;
use std::env::args;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::Path;

const LIVE_MODE: bool = false;
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
    path: String,
}

impl MarkdownLoader {
    pub fn load(&mut self) -> String {
        if self.path.is_empty() {
            return self.cache.clone();
        }

        if !self.cache.is_empty() && !LIVE_MODE {
            self.cache
                .clone()
                .lines()
                .map(|s| format!("    {s}\n"))
                .collect::<String>()
        } else {
            if LIVE_MODE {
                info!("Live mode is on. Regenerating HTML");
            } else {
                info!("Regenerating HTML");
            }

            self.cache =
                file_to_html(Path::new(&self.path)).expect("Failed to load the Markdown file!");

            self.cache
                .clone()
                .lines()
                .map(|s| format!("    {s}\n"))
                .collect::<String>()
        }
    }

    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub fn get_page_name(&self) -> String {
        let file_name = Path::new(&self.path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_lowercase();

        let mut chars = file_name.chars();
        chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str()
    }
}

fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();

    let mut markdown_loader = MarkdownLoader::default();
    let args: Vec<String> = args().collect();

    if args.len() > 1 {
        if !Path::new(&args[1]).is_file() {
            error!("Please specify a valid Markdown file!");
            return;
        };

        markdown_loader.set_path(args[1].clone());
    } else {
        error!("Please specify a Markdown file!");
        return;
    }

    let listener = TcpListener::bind("0.0.0.0:7250").unwrap();

    info!("Initialised. Listening on `localhost:7250`");
    info!("Page name: {}", markdown_loader.get_page_name());

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_request(stream, &mut markdown_loader);
    }
}

fn handle_request(mut stream: TcpStream, markdown_loader: &mut MarkdownLoader) {
    info!("Connection established");

    let status = "HTTP/1.1 200 OK";
    let data = format!(
        "<!DOCTYPE html>\n<head>{}</head>\n\n<body>\n{}</body>",
        CSS,
        markdown_loader.load()
    );
    let length = data.len();

    let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{data}");

    stream
        .write_all(response.as_bytes())
        .expect("Failed to write to stream TCP.");
}
