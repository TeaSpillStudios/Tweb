#![allow(unused_assignments)]

const LIVE_MODE: bool = false;
const LOG_IPS: bool = true;
const CSS: &str = include_str!("styles.css");

mod markdown_loader;

use chrono::Utc;
use log::{error, info, LevelFilter};
use markdown_loader::MarkdownLoader;
use std::env::args;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::path::Path;

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
                    "
                    <!DOCTYPE html>
                    <head>
                        <title>{}</title>
                        <meta charset='utf-8'>
                        {}
                    </head>

                    <body>
                        {}
                    </body>",
                    markdown_loader.get_page_name(get),
                    CSS,
                    markdown_loader.load_page(get)
                );
            } else {
                data = format!(
                    "
                <!DOCTYPE html>
                <head>
                    <title>Page not found</title>
                    <meta charset='utf-8'>
                    {}
                </head>

                <body>
                    <h1>Page not found.</h1>
                    <p>Status code: 404</p>
                </body>",
                    CSS
                )
                .to_string();
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
