use crate::{journal, storage};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub type Storage = Arc<Mutex<HashMap<String, String>>>;

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
    map.insert(key.to_string(), value.to_string());
}

pub fn set(key: &str, value: &str) {
    set_internal(key, value);

    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_set(key, value);
    }
}

pub fn get(key: &str) -> Option<String> {
    let map = get_storage().lock().unwrap();
    map.get(key).cloned()
}

pub fn exists(key: &str) -> bool {
    let map = get_storage().lock().unwrap();
    map.contains_key(key)
}

pub fn remove_internal(key: &str) -> Option<String> {
    let mut map = get_storage().lock().unwrap();
    map.remove(key)
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
        let pairs: Vec<(&str, &str)> = parts[1..].chunks(2).map(|chunk| (chunk[0], chunk[1])).collect();
        journal.lock().unwrap().log_mset(&pairs);
    }
}

pub fn add_internal(key: &str, increase: i64) -> Result<i64, &'static str> {
    match get(key) {
        None => Err("ERR key not found"),
        Some(value) => match value.parse::<i64>() {
            Err(_) => Err("ERR value is not an integer"),
            Ok(num) => {
                let new_value = num + increase;
                set_internal(key, &new_value.to_string());
                Ok(new_value)
            }
        }
    }
}

pub fn increment(key:&str) -> Result<i64, &'static str> {
    let result = add_internal(key, 1);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_increment(&key);
    }
    result
}

pub fn decrement(key:&str) -> Result<i64, &'static str>{
    let result = add_internal(key, -1);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_decrement(&key);
    }
    result
}

pub fn add(key: &str, num: i64) -> Result<i64, &'static str>{
    let result = add_internal(key, num);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_add(&key,num);
    }
    result
}

pub fn subtract(key: &str, num: i64)  -> Result<i64, &'static str>{
    let result = add_internal(key, -num);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_subtract(&key,num);
    }
    result
}

pub fn rename_internal(from: &str, to: &str)->bool {
    match remove_internal(from) {
        Some(value) => {
            set_internal(to, &value);
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