use crate::{helpers, storage};
use helpers::get_timestamp;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::sync::{Mutex, OnceLock};

pub struct Journal {
    writer: BufWriter<File>,
}

impl Journal {
    fn new(path: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        Journal {
            writer: BufWriter::new(file),
        }
    }

    pub fn log_set(&mut self, key: &str, value: &str) {
        writeln!(self.writer, "{} SET {} {}", get_timestamp(), key, value).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_remove(&mut self, key: &str) {
        writeln!(self.writer, "{} DEL {}", get_timestamp(), key).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_rename(&mut self, from: &str, to: &str) {
        writeln!(self.writer, "{} RENAME {} {}", get_timestamp(), from, to).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_increment(&mut self, key: &str) {
        writeln!(self.writer, "{} INCR {}", get_timestamp(), key).unwrap();
        self.writer.flush().unwrap();
    }
    pub fn log_decrement(&mut self, key: &str) {
        writeln!(self.writer, "{} DECR {}", get_timestamp(), key).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_mset(&mut self, pairs: &[(&str, &str)]) {
        let flat: Vec<&str> = pairs.iter().flat_map(|(k, v)| [*k, *v]).collect();
        writeln!(self.writer, "{} MSET {}", get_timestamp(), flat.join(" ")).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_add(&mut self, key: &str, value: i64) {
        writeln!(self.writer, "{} ADD {} {}", get_timestamp(), key, value).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_subtract(&mut self, key: &str, value: i64) {
        writeln!(self.writer, "{} SUB {} {}", get_timestamp(), key, value).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_expire(&mut self, key: &str, seconds: u64) {
        writeln!(
            self.writer,
            "{} EXPIRE {} {}",
            get_timestamp(),
            key,
            seconds
        )
        .unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_pushr(&mut self, key: &str, value: &str) {
        writeln!(self.writer, "{} PUSHR {} {}", get_timestamp(), key, value).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_pushl(&mut self, key: &str, value: &str) {
        writeln!(self.writer, "{} PUSHL {} {}", get_timestamp(), key, value).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_popr(&mut self, key: &str) {
        writeln!(self.writer, "{} POPR {}", get_timestamp(), key).unwrap();
        self.writer.flush().unwrap();
    }
    pub fn log_popl(&mut self, key: &str) {
        writeln!(self.writer, "{} POPL {}", get_timestamp(), key).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_sadd(&mut self, key: &str, member: &str) {
        writeln!(self.writer, "{} SADD {} {}", get_timestamp(), key, member,).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_srem(&mut self, key: &str, member: &str) {
        writeln!(self.writer, "{} SREM {} {}", get_timestamp(), key, member,).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_hset(&mut self, key: &str, field: &str, value: &str) {
        writeln!(
            self.writer,
            "{} HSET {} {} {}",
            get_timestamp(),
            key,
            field,
            value
        )
        .unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_hdel(&mut self, key: &str, field: &str) {
        writeln!(self.writer, "{} HDEL {} {}", get_timestamp(), key, field,).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn clear_journal(&mut self) {
        self.writer.get_mut().set_len(0).unwrap();
        self.writer.get_mut().seek(SeekFrom::Start(0)).unwrap();
        self.writer.flush().unwrap();
    }
}

pub static JOURNAL: OnceLock<Mutex<Journal>> = OnceLock::new();

pub fn init_journal(path: &str) {
    JOURNAL.get_or_init(|| Mutex::new(Journal::new(path)));
    replay_journal(path);
}

fn replay_journal(path: &str) {
    let file = File::open(path);
    if file.is_err() {
        return; //
    }

    let reader = BufReader::new(file.unwrap());
    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.splitn(4, ' ').collect();

        if parts.len() < 3 {
            continue;
        }

        match parts[1] {
            "SET" if parts.len() == 4 => storage::set_internal(parts[2], parts[3]),
            "DEL" if parts.len() == 3 => {
                let _ = storage::remove_internal(parts[2]);
            }
            "RENAME" if parts.len() == 4 => {
                let _ = storage::rename_internal(parts[2], parts[3]);
            }
            "MSET" if parts.len() >= 3 => {
                let all: Vec<&str> = line.split(' ').collect();
                // all = [timestamp, "MSET", key1, val1, key2, val2, ...]
                for pair in all[2..].chunks(2) {
                    if pair.len() == 2 {
                        storage::set_internal(pair[0], pair[1]);
                    }
                }
            }
            "INCR" if parts.len() == 3 => {
                let _ = storage::add_internal(parts[2], 1);
            }
            "DECR" if parts.len() == 3 => {
                let _ = storage::add_internal(parts[2], -1);
            }
            "ADD" if parts.len() == 4 => {
                let value = parts[3].parse::<i64>();
                match value {
                    Ok(num) => {
                        let _ = storage::add_internal(parts[2], num);
                    }
                    Err(_) => {}
                }
            }
            "SUB" if parts.len() == 4 => {
                let value = parts[3].parse::<i64>();
                match value {
                    Ok(num) => {
                        let _ = storage::add_internal(parts[2], -num);
                    }
                    Err(_) => {}
                }
            }
            "EXPIRE" if parts.len() == 4 => {
                let value = parts[2].parse::<u64>();
                match value {
                    Ok(seconds) => {
                        let _ = storage::expire_interal(parts[2], seconds);
                    }
                    _ => {}
                }
            }
            "PUSHR" if parts.len() == 4 => {
                let _ = storage::push_right_internal(parts[2], parts[3]);
            }
            "PUSHL" if parts.len() == 4 => {
                let _ = storage::push_left_internal(parts[2], parts[3]);
            }
            "POPR" if parts.len() == 3 => {
                let _ = storage::pop_right_internal(parts[2]);
            }
            "POPL" if parts.len() == 3 => {
                let _ = storage::pop_left_internal(parts[2]);
            }
            "SADD" if parts.len() == 4 => {
                let _ = storage::set_add_internal(parts[2], parts[3]);
            }
            "SREM" if parts.len() == 3 => {
                let _ = storage::set_remove_internal(parts[2], parts[3]);
            }
            "HSET" if parts.len() >= 4 => {
                let all: Vec<&str> = line.split(' ').collect();
                // all = [timestamp, "HSET", key, field, value...]
                if all.len() >= 5 {
                    let value = all[4..].join(" "); // multi-word value support
                    storage::hash_set_internal(all[2], all[3], &value);
                }
            }
            "HDEL" if parts.len() == 4 => {
                let _ = storage::hash_delete_internal(parts[2], parts[3]);
            }
            _ => {}
        }
    }
}
