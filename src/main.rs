mod command;
mod config;
mod helpers;
mod journal;
mod storage;
mod protocol;
mod datagram;

use command::handle_command;
use config::Config;
use journal::init_journal;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(stream: TcpStream) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();

    println!("New client connected: {}", stream.peer_addr().unwrap());
    writer.write_all(b"Blitz v0.1\r\n").unwrap();
    loop {
        line.clear();
        writer.write_all(b"> ").unwrap();
        match reader.read_line(&mut line) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(_) => {
                // print!("Received: {}", line);
                if handle_command(&mut writer, &line) {
                    break;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
}

fn main() {
    let config = Config::new();
    let listener = TcpListener::bind(config.address()).unwrap();
    init_journal(&*config.journal_path);
    println!("Blitz listening on {}:{}", config.host, config.port);

    std::thread::spawn(|| {
        datagram::start_server();
    });

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        std::thread::spawn(|| {
            handle_client(stream);
        });
    }
}
