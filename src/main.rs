use chrono::Utc;
use log::{error, info, LevelFilter};
use markdown::file_to_html;
use std::env::args;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::{IpAddr, TcpListener, TcpStream};
use std::path::Path;

const LIVE_MODE: bool = false;
const LOG_IPS: bool = true;
const CSS: &str = include_str!("styles.css");

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
            info!("Serving from cache.");

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

    pub fn get_page_name(&mut self) -> String {
        let markdown_title = fs::read_to_string(&self.path).unwrap();

        markdown_title
            .lines()
            .next()
            .unwrap()
            .split_once(' ')
            .unwrap()
            .1
            .to_string()
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

    info!("Initialised. Listening on `http://localhost:7250`");
    info!("Page name: {}", markdown_loader.get_page_name());

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_request(stream, &mut markdown_loader);
    }
}

fn handle_request(mut stream: TcpStream, markdown_loader: &mut MarkdownLoader) {
    info!(
        "Connection established: {}",
        stream.peer_addr().unwrap().ip()
    );

    if LOG_IPS {
        log_ip(stream.peer_addr().unwrap().ip());
    }

    let status = "HTTP/1.1 200 OK";
    let data = format!(
        "<!DOCTYPE html>\n<head>\n    <title>{}</title>\n{}</head>\n\n<body>\n{}</body>",
        markdown_loader.get_page_name(),
        CSS,
        markdown_loader.load()
    );
    let length = data.len();

    let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{data}");

    stream
        .write_all(response.as_bytes())
        .expect("Failed to write to stream TCP.");
}

fn log_ip(ip: IpAddr) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("./log.md")
        .unwrap();

    let date = Utc::now().date_naive().to_string();
    let data = format!("## {date}\n    {ip}\n\n");

    file.write_all(data.as_bytes()).unwrap();
}
