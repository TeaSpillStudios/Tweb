use chrono::Utc;
use log::{error, info, LevelFilter};
use markdown::file_to_html;
use std::collections::HashMap;
use std::env::args;
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::path::Path;

const LIVE_MODE: bool = true;
const LOG_IPS: bool = true;
const MULTIPAGE: bool = true;
const CSS: &str = include_str!("styles.css");

#[derive(Default)]
struct MarkdownLoader {
    cache: HashMap<String, String>,
    root_path: String,
}

impl MarkdownLoader {
    pub fn load_page(&mut self, page_name: &str) -> String {
        if self.root_path.is_empty() {
            return self.cache.get(page_name).unwrap().to_owned();
        }

        if self.cache.contains_key(page_name) && !LIVE_MODE {
            info!("Serving from cache.");

            self.cache
                .get(page_name)
                .unwrap()
                .lines()
                .map(|s| format!("    {s}\n"))
                .collect::<String>()
        } else {
            if LIVE_MODE {
                info!("Live mode is on. Regenerating HTML");
            } else {
                info!("Regenerating HTML");
            }

            if page_name == "" && !MULTIPAGE {
                self.cache.insert(
                    String::from(page_name),
                    file_to_html(Path::new(&self.root_path))
                        .expect("Failed to load the Markdown file!"),
                );
            } else {
                self.cache.insert(
                    String::from(page_name),
                    file_to_html(Path::new(page_name))
                        .expect("Failed to load the specified Markdown file!"),
                );
            }

            self.cache
                .get(page_name)
                .unwrap()
                .lines()
                .map(|s| format!("    {s}\n"))
                .collect::<String>()
        }
    }

    pub fn set_path(&mut self, path: String) {
        self.root_path = path;
    }

    pub fn get_page_name(&mut self) -> String {
        let markdown_title = fs::read_to_string(&self.root_path).unwrap();

        markdown_title
            .lines()
            .next()
            .unwrap_or("Default title")
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
    let peer_address = stream.peer_addr().unwrap().ip();

    info!("Connection established: {peer_address}",);

    if LOG_IPS {
        log_ip(peer_address);
    }

    let http_request: Vec<_> = BufReader::new(&mut stream)
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    for line in http_request {
        if line.contains("HTTP/1.1") {
            let get = line.split(' ').collect::<Vec<&str>>()[1]
                .split('/')
                .collect::<Vec<&str>>()[1];

            let status = "HTTP/1.1 200 OK";
            let data = format!(
                "<!DOCTYPE html>\n<head>\n    <title>{}</title>\n{}</head>\n\n<body>\n{}</body>",
                markdown_loader.get_page_name(),
                CSS,
                markdown_loader.load_page(get)
            );
            let length = data.len();

            let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{data}");

            stream
                .write_all(response.as_bytes())
                .expect("Failed to write to stream TCP.");

            dbg!(get);
        }
    }
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
