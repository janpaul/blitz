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