#![allow(unused_assignments)]

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

const LIVE_MODE: bool = false;
const LOG_IPS: bool = true;
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

            if page_name == "" {
                self.cache.insert(
                    String::from(page_name),
                    file_to_html(Path::new(&self.root_path))
                        .expect("Failed to load the Markdown file!"),
                );
            } else {
                let page_file_name = match page_name.ends_with(".md") {
                    true => page_name.to_string(),
                    false => page_name.to_string() + ".md",
                };
                self.cache.insert(
                    page_name.to_string(),
                    file_to_html(Path::new(&page_file_name)).expect(&format!(
                        "Failed to load the specified Markdown file `{page_name}`!"
                    )),
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

    pub fn validate_page(&self, page_name: &str) -> bool {
        let page_file_name = match page_name.ends_with(".md") {
            true => page_name.to_string(),
            false => page_name.to_string() + ".md",
        };

        match page_name == "" {
            true => true,
            false => Path::new(&page_file_name).is_file(),
        }
    }

    pub fn set_path(&mut self, path: String) {
        self.root_path = path;
    }

    pub fn get_page_name(&mut self, page_name: &str) -> String {
        let mut page_file_name = String::new();

        if page_name != "" {
            page_file_name = match page_name.ends_with(".md") {
                true => page_name.to_string(),
                false => page_name.to_string() + ".md",
            };
        } else {
            page_file_name = self.root_path.clone();
        }

        let markdown_title = fs::read_to_string(page_file_name).unwrap();

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

            let ok = markdown_loader.validate_page(get);

            let status = match ok {
                true => "HTTP/1.1 200 OK",
                false => "HTTP/1.1 404 PAGE_NOT_FOUND",
            };

            let mut data = String::new();

            if ok {
                data = format!(
                    "<!DOCTYPE html>\n<head>\n    <title>{}</title>\n{}</head>\n\n<body>\n{}</body>",
                    markdown_loader.get_page_name(get),
                    CSS,
                    markdown_loader.load_page(get)
                );
            } else {
                data = format!("<!DOCTYPE html>\n<head>\n    <title>Page not found</title>\n{}</head>\n\n<body>\n<h1>Page not found.</h1><p>Status code: 404</p></body>", CSS).to_string();
            }

            let length = data.len();

            let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{data}");

            stream
                .write_all(response.as_bytes())
                .expect("Failed to write to stream TCP.");
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
