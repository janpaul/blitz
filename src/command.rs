use crate::storage;
use std::io::Write;

const NIL:&str = "nil\r\n";
const OK:&str = "OK\r\n";
const NOK:&str = "NOK\r\n";
const BYE:&str = "BYE\r\n";

fn write_response<W: Write>(writer: &mut W, response: &str) {
    writer.write_all(response.as_bytes()).unwrap();
}

fn handle_set<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() < 3 {
        write_response(writer, NOK);
        return;
    }

    let value = parts[2..].join(" ");
    storage::set(parts[1], &value);
    write_response(writer, OK);
}

fn handle_get<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() < 2 {
        write_response(writer, NOK);
        return;
    }
    match storage::get(parts[1]) {
        Some(value) => write_response(writer, &format!("{}\r\n", value)),
        None => write_response(writer, NIL),
    }
}

fn handle_list<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 1 {
        write_response(writer, NOK);
    }
    let keys = storage::get_keys();
    if keys.is_empty() {
        write_response(writer, NIL);
    } else {
        let response = keys.join("\n") + "\r\n";
        write_response(writer, &response);
    }
}

fn handle_clear<W: Write>(writer: &mut W) {
    storage::clear();
    write_response(writer, OK);
}

fn handle_delete<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 2 {
        write_response(writer, NOK);
    }

    match  storage::remove(parts[1]) {
        Some(key) => write_response(writer, &format!("{}\r\n", key)),
        None => write_response(writer, NIL),
    }
}

fn handle_help<W: Write>(writer: &mut W) {
    write_response(writer, "SET <key> <value>\r\n");
    write_response(writer, "GET <key>\r\n");
    write_response(writer, "DEL <key>\r\n");
    write_response(writer, "LIST");
    write_response(writer, "CLEAR");
    write_response(writer, "QUIT");
}

pub fn handle_command<W:Write>(writer: &mut W, command: &str) -> bool {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();

    if parts.is_empty() {
        return false;
    }

    match parts[0].to_uppercase().as_str() {
        "SET" => handle_set(writer, &parts),
        "GET" => handle_get(writer, &parts),
        "LIST" => handle_list(writer, &parts),
        "CLEAR" => handle_clear(writer),
        "DEL" => handle_delete(writer, &parts),
        "HELP" => handle_help(writer),
        "QUIT" => {
            write_response(writer,BYE);
            return false
        },
        _ => {
            write_response(writer, NOK);
            println!("Unknown command: {}", parts[0]);
        }
    }

    false
}