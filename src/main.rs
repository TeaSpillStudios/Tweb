#![allow(unused_assignments)]

const LIVE_MODE: bool = false;
const LOG_IPS: bool = true;
const CSS: &str = include_str!("styles.css");

mod html_composer;
mod markdown_loader;

use chrono::Utc;
use log::{error, info, LevelFilter};
use markdown_loader::MarkdownLoader;
use std::env::args;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::path::Path;

use crate::html_composer::compose_html;

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

            stream
                .write_all(compose_html(get, markdown_loader).as_bytes())
                .unwrap()
        };
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
