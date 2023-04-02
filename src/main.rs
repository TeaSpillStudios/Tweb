#![allow(unused_assignments)]

const LIVE_MODE: bool = false;
const LOG_IPS: bool = true;
const CSS: &str = include_str!("styles.css");

mod html_composer;
mod markdown_loader;

use chrono::Utc;
use chunked_transfer::Encoder;
use log::{error, info, warn, LevelFilter};
use markdown_loader::MarkdownLoader;
use std::env::args;
use std::fs::{create_dir, File, OpenOptions};
use std::io::Read;
use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::path::Path;

use crate::html_composer::compose_html;

const FILE_WHITELSIT: [&str; 1] = ["favicon.ico"];

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

            if FILE_WHITELSIT.contains(&get) {
                info!("Sending a normal file: {get}");

                let mut file = match File::open(Path::new(get)) {
                    Ok(v) => v,
                    Err(_) => {
                        warn!("Could not access file!");
                        return;
                    }
                };
                let mut buf = Vec::new();

                file.read_to_end(&mut buf).unwrap();

                let mut encoded = Vec::new();

                {
                    let mut encoder = Encoder::with_chunks_size(&mut encoded, 8);
                    encoder.write_all(&buf).unwrap()
                }

                let headers = [
                    "HTTP/1.1 200 OK",
                    "Content-type: image/jpeg",
                    "Transfer-Encoding: chunked",
                    "\r\n",
                ];
                let mut response = headers.join("\r\n").to_string().into_bytes();
                response.extend(encoded);

                stream.write_all(&response).unwrap();
            } else {
                stream
                    .write_all(compose_html(get, markdown_loader).as_bytes())
                    .unwrap()
            }
        };
    }
}

fn log_ip(ip: IpAddr) {
    let data_dir = match dirs::data_dir() {
        Some(v) => {
            create_dir(v.join("tweb")).ok();
            v
        }

        None => {
            error!("Cannot open log path!");
            return;
        }
    };

    let log_path = data_dir.join(Path::new("tweb/log.md"));

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_path)
        .unwrap();

    let date = Utc::now().date_naive().to_string();
    let data = format!("## {date}\n    {ip}\n\n");

    file.write_all(data.as_bytes()).unwrap();
}
