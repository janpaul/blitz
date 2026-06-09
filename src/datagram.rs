use std::net::UdpSocket;
use crate::protocol::*;
use crate::storage;

use crate::config::Config;

pub fn start_server() {
    let config = Config::new();
    let socket = UdpSocket::bind(config.datagram_address()).unwrap();
    println!("Blitz UDP listening on port 6380");

    let mut buf = [0u8; 513];

    loop {
        let (len, src) = socket.recv_from(&mut buf).unwrap();
        let packet = &buf[..len];
        if packet.is_empty() {
            continue;
        }
        let response = handle_packet(packet);
        socket.send_to(&response, src).unwrap();
    }
}

fn handle_packet(packet: &[u8]) -> Vec<u8> {
    match packet[0] {
        CMD_GET => handle_get(packet),
        CMD_SET => handle_set(packet),
        _ => vec![STATUS_ERR],
    }
}

fn handle_get(packet: &[u8]) -> Vec<u8> {
    if packet.len() < 2 {
        return vec![STATUS_ERR];
    }
    let key_len = packet[1] as usize;
    if packet.len() < 2 + key_len {
        return vec![STATUS_ERR];
    }
    let key = std::str::from_utf8(&packet[2..2 + key_len]).unwrap_or("");
    match storage::get(key) {
        None => vec![STATUS_NIL],
        Some(value) => {
            let bytes = value.as_bytes();
            let mut response = vec![STATUS_OK, bytes.len() as u8];
            response.extend_from_slice(bytes);
            response
        }
    }
}
fn handle_set(packet: &[u8]) -> Vec<u8> {
    if packet.len() < 3 {
        return vec![STATUS_ERR];
    }
    let key_len = packet[1] as usize;
    if packet.len() < 2 + key_len + 1 {
        return vec![STATUS_ERR];
    }
    let key = match std::str::from_utf8(&packet[2..2 + key_len]) {
        Ok(k) => k,
        Err(_) => return vec![STATUS_ERR],
    };
    let value_len = packet[2 + key_len] as usize;
    if packet.len() < 2 + key_len + 1 + value_len {
        return vec![STATUS_ERR];
    }
    let value = match std::str::from_utf8(&packet[3 + key_len..3 + key_len + value_len]) {
        Ok(v) => v,
        Err(_) => return vec![STATUS_ERR],
    };
    storage::set(key, value);
    vec![STATUS_OK]
}