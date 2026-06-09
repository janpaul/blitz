use std::fs::{OpenOptions,File};
use std::io::{BufWriter, BufReader, BufRead,Write,Seek,SeekFrom};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::storage;


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

    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn log_set(&mut self, key: &str, value: &str) {
        writeln!(self.writer, "{} SET {} {}", Self::get_timestamp(), key, value).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_remove(&mut self, key: &str) {
        writeln!(self.writer, "{} DEL {}", Self::get_timestamp(), key).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_rename(&mut self, from: &str, to:&str) {
        writeln!(self.writer, "{} RENAME {} {}", Self::get_timestamp(), from, to).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_increment(&mut self, key: &str) {
        writeln!(self.writer, "{} INCR {}", Self::get_timestamp(), key).unwrap();
        self.writer.flush().unwrap();
    }
    pub fn log_decrement(&mut self, key: &str) {
        writeln!(self.writer, "{} DECR {}", Self::get_timestamp(), key).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_mset(&mut self, pairs: &[(&str, &str)]) {
        let flat: Vec<&str> = pairs.iter().flat_map(|(k, v)| [*k, *v]).collect();
        writeln!(self.writer, "{} MSET {}", Self::get_timestamp(), flat.join(" ")).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_add(&mut self, key: &str, value: i64) {
        writeln!(self.writer, "{} ADD {} {}", Self::get_timestamp(), key, value).unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_subtract(&mut self, key: &str, value: i64) {
        writeln!(self.writer, "{} SUB {} {}", Self::get_timestamp(), key, value).unwrap();
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
            "SET" if parts.len() == 4 => {
                storage::set_internal(parts[2], parts[3])
            }
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
            "INCR" if parts.len()==3 => {
                let _ = storage::add_internal(parts[2], 1);
            }
            "DECR" if parts.len()==3 => {
                let _= storage::add_internal(parts[2], -1);
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
                        let _ =storage::add_internal(parts[2], -num);
                    }
                    Err(_) => {}
                }
            }
            _ => {}
        }
    }
}


