use crate::helpers::get_timestamp;
use crate::storage;
use std::io::Write;

const NIL: &str = "nil\r\n";
const OK: &str = "OK\r\n";
const NOK: &str = "NOK\r\n";
const BYE: &str = "BYE\r\n";

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

fn handle_exists<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() < 2 {
        write_response(writer, NOK);
        return;
    }
    if storage::exists(parts[1]) {
        write_response(writer, "1\r\n");
    } else {
        write_response(writer, "0\r\n");
    }
}

fn handle_keys<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 1 {
        write_response(writer, NOK);
        return;
    }
    let keys = storage::get_keys();
    if keys.is_empty() {
        write_response(writer, NIL);
    } else {
        let response = keys
            .iter()
            .map(|(key, kind)| format!("{} [{}]", key, kind))
            .collect::<Vec<String>>()
            .join("\n")
            + "\r\n";
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

    match storage::remove(parts[1]) {
        Some(key) => write_response(writer, &format!("{}\r\n", key)),
        None => write_response(writer, NIL),
    }
}

fn handle_rename<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() < 3 {
        write_response(writer, NOK);
    }

    if storage::rename(parts[1], parts[2]) {
        write_response(writer, OK);
    } else {
        write_response(writer, format!("NOK no such key {}\r\n", parts[1]).as_str());
    }
}

fn handle_type<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 2 {
        write_response(writer, NOK);
        return;
    }

    if storage::exists(parts[1]) {
        let value = storage::get(parts[1]).unwrap();
        if value.parse::<i64>().is_ok() {
            write_response(writer, "number\r\n");
        } else {
            write_response(writer, "string\r\n");
        }
    } else if storage::list_exists(parts[1]) {
        write_response(writer, "list\r\n");
    } else {
        write_response(writer, NIL);
    }
}

fn handle_mget<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() < 2 {
        write_response(writer, NOK);
        return;
    }

    for key in &parts[1..] {
        match storage::get(key) {
            Some(value) => write_response(writer, &format!("{}\r\n", value)),
            None => write_response(writer, NIL),
        }
    }
}

fn handle_mset<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() < 3 || parts[1..].len() % 2 != 0 {
        write_response(writer, NOK);
        return;
    }
    storage::mset(parts);
    write_response(writer, OK);
}

fn handle_incr<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 2 {
        write_response(writer, NOK);
        return;
    }
    match storage::increment(parts[1]) {
        Ok(new_value) => write_response(writer, &format!("{}\r\n", new_value)),
        Err(e) => write_response(writer, &format!("{}\r\n", e)),
    }
}

fn handle_decr<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 2 {
        write_response(writer, NOK);
        return;
    }
    match storage::decrement(parts[1]) {
        Ok(new_value) => write_response(writer, &format!("{}\r\n", new_value)),
        Err(e) => write_response(writer, &format!("{}\r\n", e)),
    }
}

fn handle_add<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 3 {
        write_response(writer, NOK);
        return;
    }
    if let Ok(num) = parts[2].parse::<i64>() {
        match storage::add(parts[1], num) {
            Ok(new_value) => write_response(writer, &format!("{}\r\n", new_value)),
            Err(e) => write_response(writer, &format!("{}\r\n", e)),
        }
        write_response(writer, OK);
    } else {
        write_response(writer, NOK);
    }
}

fn handle_sub<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 3 {
        write_response(writer, NOK);
        return;
    }
    if let Ok(num) = parts[2].parse::<i64>() {
        match storage::subtract(parts[1], num) {
            Ok(new_value) => write_response(writer, &format!("{}\r\n", new_value)),
            Err(e) => write_response(writer, &format!("{}\r\n", e)),
        }
    } else {
        write_response(writer, NOK);
    }
}

fn handle_expire<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 3 {
        write_response(writer, NOK);
        return;
    }

    if let Ok(num) = parts[2].parse::<u64>() {
        if storage::expire(parts[1], num) {
            write_response(writer, OK);
        } else {
            write_response(writer, format!("NOK no such key {}\r\n", parts[1]).as_str());
        }
    } else {
        write_response(writer, NOK);
    }
}

fn handle_ttl<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 2 {
        write_response(writer, NOK);
        return;
    }
    match storage::ttl(parts[1]) {
        Some(ttl) => write_response(writer, &format!("{}\r\n", ttl - get_timestamp())),
        None => write_response(writer, NIL),
    }
}

fn handle_pushr<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() < 3 {
        write_response(writer, NOK);
        return;
    }
    let value = parts[2..].join(" ");
    storage::push_right(parts[1], &value);
    write_response(writer, OK);
}

fn handle_pushl<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() < 3 {
        write_response(writer, NOK);
        return;
    }
    let value = parts[2..].join(" ");
    storage::push_left(parts[1], &value);
    write_response(writer, OK);
}

fn handle_popr<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 2 {
        write_response(writer, NOK);
        return;
    }
    match storage::pop_right(parts[1]) {
        Some(value) => write_response(writer, &format!("{}\r\n", value)),
        None => write_response(writer, NIL),
    }
}

fn handle_popl<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 2 {
        write_response(writer, NOK);
        return;
    }
    match storage::pop_left(parts[1]) {
        Some(value) => write_response(writer, &format!("{}\r\n", value)),
        None => write_response(writer, NIL),
    }
}

fn handle_llen<W: Write>(writer: &mut W, parts: &[&str]) {
    if parts.len() != 2 {
        write_response(writer, NOK);
        return;
    }
    write_response(writer, &format!("{}\r\n", storage::llen(parts[1])));
}

pub fn handle_command<W: Write>(writer: &mut W, command: &str) -> bool {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();

    if parts.is_empty() {
        return false;
    }

    match parts[0].to_uppercase().as_str() {
        // key/value functions
        "SET" => handle_set(writer, &parts),
        "GET" => handle_get(writer, &parts),
        "LIST" | "LS" | "KEYS" => handle_keys(writer, &parts),
        "EXISTS" => handle_exists(writer, &parts),
        "RENAME" | "REN" | "MOVE" => handle_rename(writer, &parts),
        "TYPE" => handle_type(writer, &parts),
        "MGET" => handle_mget(writer, &parts),
        "MSET" => handle_mset(writer, &parts),
        "INCR" => handle_incr(writer, &parts),
        "DECR" => handle_decr(writer, &parts),
        "ADD" => handle_add(writer, &parts),
        "SUB" | "SUBTRACT" => handle_sub(writer, &parts),
        "EXPIRE" | "EXP" => handle_expire(writer, &parts),
        "TTL" => handle_ttl(writer, &parts),
        // list functions
        "PUSHR" => handle_pushr(writer, &parts),
        "PUSHL" => handle_pushl(writer, &parts),
        "POPL" => handle_popl(writer, &parts),
        "POPR" => handle_popr(writer, &parts),
        "LLEN" => handle_llen(writer, &parts),
        // common functions
        "CLEAR" => handle_clear(writer),
        "DEL" => handle_delete(writer, &parts),
        "HELP" => handle_help(writer),
        "PING" => write_response(writer, "PONG\r\n"),
        "QUIT" | "BYE" => {
            write_response(writer, BYE);
            return true;
        }
        _ => {
            write_response(writer, NOK);
            println!("Unknown command: {}", parts[0]);
        }
    }

    false
}

fn handle_help<W: Write>(writer: &mut W) {
    write_response(writer, "SET <key> <value>\r\n");
    write_response(writer, "GET <key>\r\n");
    write_response(writer, "DEL <key>\r\n");
    write_response(writer, "EXISTS <key>\r\n");
    write_response(writer, "TYPE <key>\r\n");
    write_response(writer, "REN <old> <new>\r\n");
    write_response(writer, "MOVE <old> <new>\r\n");
    write_response(writer, "RENAME <old> <new>\r\n");
    write_response(writer, "INCR <key>\r\n");
    write_response(writer, "DECR <key>\r\n");
    write_response(writer, "ADD <key> <number>\r\n");
    write_response(writer, "SUB <key> <number>\r\n");
    write_response(writer, "SUBTRACT <key> <number>\r\n");
    write_response(writer, "EXPIRE <key> <seconds>\r\n");
    write_response(writer, "TTL <key>\r\n");
    write_response(writer, "MGET <key1> <key2> ... <keyn>\r\n");
    write_response(
        writer,
        "MSET <key1> <value1> <key2> <value2>... <keyn> <valuen>\r\n",
    );
    write_response(writer, "PUSHR <key> <value>\r\n");
    write_response(writer, "PUSHL <key> <value>\r\n");
    write_response(writer, "POPR <key>\r\n");
    write_response(writer, "POPL <key>\r\n");
    write_response(writer, "LLEN <key>\r\n");
    write_response(writer, "PING\r\n");
    write_response(writer, "LS\r\n");
    write_response(writer, "LIST\r\n");
    write_response(writer, "CLEAR\r\n");
    write_response(writer, "QUIT\r\n");
    write_response(writer, "BYE\r\n");
}
