use crate::journal;
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