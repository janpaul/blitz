use crate::{helpers, journal, storage};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use helpers::get_timestamp;

struct Entry {
    value: String,
    expires_at: Option<u64>, // Unix timestamp, None = nooit
}

impl Entry {
    fn new(value: String, expires_at: Option<u64>) -> Self {
        Entry { value, expires_at }
    }
}
type Storage = Arc<Mutex<HashMap<String, Entry>>>;

static STORAGE: OnceLock<Storage> = OnceLock::new();

fn get_storage() -> &'static Storage {
    STORAGE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

pub fn get_keys() -> Vec<String> {
    let map = get_storage().lock().unwrap();
    map.keys().cloned().collect()
}

pub fn clear() {
    get_storage().lock().unwrap().clear();

    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().clear_journal();
    }
}

pub fn set_internal(key: &str, value: &str) {
    let mut map = get_storage().lock().unwrap();
    map.insert(key.to_string(), Entry::new(value.to_string(), None));
}

pub fn set(key: &str, value: &str) {
    set_internal(key, value);

    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_set(key, value);
    }
}

pub fn expire_interal(key:&str, seconds: u64) -> bool{
    let mut map = get_storage().lock().unwrap();
    match map.get_mut(key) {
        Some(entry) => {
            entry.expires_at = Some(get_timestamp() + seconds);
        }
        None => return false,
    }
    true
}

pub fn expire(key:&str, seconds: u64) -> bool{
    expire_interal(key, seconds);

    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_expire(key, seconds);
    }
    true
}

pub fn ttl(key:&str) -> Option<u64> {
    let map = get_storage().lock().unwrap();
    match map.get(key) {
        Some(entry) => {
            entry.expires_at
        }
        None => None,
    }
}

pub fn get(key: &str) -> Option<String> {
    let mut map = get_storage().lock().unwrap();
    match map.get(key) {
        Some(entry) => {
            if let Some(expires_at) = entry.expires_at {
                if expires_at < get_timestamp() {
                    map.remove(key);
                    return None;
                }
            }
            Some(entry.value.clone())
        }
        None => None,
    }
}

pub fn exists(key: &str) -> bool {
    let map = get_storage().lock().unwrap();
    map.contains_key(key)
}

pub fn remove_internal(key: &str) -> Option<String> {
    let mut map = get_storage().lock().unwrap();
    map.remove(key).map(|entry| entry.value)
}
pub fn remove(key: &str) -> Option<String> {
    let value = remove_internal(key);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_remove(key);
    }
    value
}

pub fn mset(parts: &[&str]) {
    for pair in parts[1..].chunks(2) {
        storage::set_internal(pair[0], pair[1]);
    }
    if let Some(journal) = journal::JOURNAL.get() {
        let pairs: Vec<(&str, &str)> = parts[1..]
            .chunks(2)
            .map(|chunk| (chunk[0], chunk[1]))
            .collect();
        journal.lock().unwrap().log_mset(&pairs);
    }
}

pub fn add_internal(key: &str, increase: i64) -> Result<i64, &'static str> {
    let mut map = get_storage().lock().unwrap();
    match map.get_mut(key) {
        None => Err("ERR key not found"),
        Some(entry) => match entry.value.parse::<i64>() {
            Err(_) => Err("ERR value is not an integer"),
            Ok(num) => {
                let new_value = num + increase;
                entry.value = new_value.to_string(); // muteer in-place, expires_at blijft intact
                Ok(new_value)
            }
        }
    }
}

pub fn increment(key: &str) -> Result<i64, &'static str> {
    let result = add_internal(key, 1);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_increment(&key);
    }
    result
}

pub fn decrement(key: &str) -> Result<i64, &'static str> {
    let result = add_internal(key, -1);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_decrement(&key);
    }
    result
}

pub fn add(key: &str, num: i64) -> Result<i64, &'static str> {
    let result = add_internal(key, num);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_add(&key, num);
    }
    result
}

pub fn subtract(key: &str, num: i64) -> Result<i64, &'static str> {
    let result = add_internal(key, -num);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_subtract(&key, num);
    }
    result
}

pub fn rename_internal(from: &str, to: &str) -> bool {
    let mut map = get_storage().lock().unwrap();
    match map.remove(from) {
        Some(entry) => {
            map.insert(to.to_string(), entry);
            true
        }
        None => false,
    }
}

pub fn rename(from: &str, to: &str) -> bool {
    let value = rename_internal(from, to);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_rename(from, to);
    }
    value
}
