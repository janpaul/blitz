use crate::{helpers, journal, storage};
use helpers::get_timestamp;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

struct Entry {
    value: String,
    expires_at: Option<u64>, // Unix timestamp, None = never
}

impl Entry {
    fn new(value: String, expires_at: Option<u64>) -> Self {
        Entry { value, expires_at }
    }
}
type ValueStorage = Arc<Mutex<HashMap<String, Entry>>>;
type ListStorage = Arc<Mutex<HashMap<String, Vec<String>>>>;
type SetStorage = Arc<Mutex<HashMap<String, HashSet<String>>>>;
type HashStorage = Arc<Mutex<HashMap<String, HashMap<String, String>>>>;
static VALUE_STORAGE: OnceLock<ValueStorage> = OnceLock::new();
static LIST_STORAGE: OnceLock<ListStorage> = OnceLock::new();
static SET_STORAGE: OnceLock<SetStorage> = OnceLock::new();
static HASH_STORAGE: OnceLock<HashStorage> = OnceLock::new();

fn get_value_storage() -> &'static ValueStorage {
    VALUE_STORAGE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

fn get_list_storage() -> &'static ListStorage {
    LIST_STORAGE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

fn get_set_storage() -> &'static SetStorage {
    SET_STORAGE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

fn get_hash_storage() -> &'static HashStorage {
    HASH_STORAGE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

pub fn get_keys() -> Vec<(String, &'static str)> {
    let value_map = get_value_storage().lock().unwrap();
    let list_map = get_list_storage().lock().unwrap();
    let set_map = get_set_storage().lock().unwrap();
    let hash_map = get_hash_storage().lock().unwrap();
    value_map
        .keys()
        .map(|k| (k.clone(), "string"))
        .chain(list_map.keys().map(|k| (k.clone(), "list")))
        .chain(set_map.keys().map(|k| (k.clone(), "set")))
        .chain(hash_map.keys().map(|k| (k.clone(), "hash")))
        .collect()
}

pub fn clear() {
    get_value_storage().lock().unwrap().clear();

    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().clear_journal();
    }
}

pub fn set_internal(key: &str, value: &str) {
    let mut map = get_value_storage().lock().unwrap();
    map.insert(key.to_string(), Entry::new(value.to_string(), None));
}

pub fn set(key: &str, value: &str) {
    set_internal(key, value);

    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_set(key, value);
    }
}

pub fn expire_interal(key: &str, seconds: u64) -> bool {
    let mut map = get_value_storage().lock().unwrap();
    match map.get_mut(key) {
        Some(entry) => {
            entry.expires_at = Some(get_timestamp() + seconds);
        }
        None => return false,
    }
    true
}

pub fn expire(key: &str, seconds: u64) -> bool {
    let value = expire_interal(key, seconds);
    if value {
        if let Some(journal) = journal::JOURNAL.get() {
            journal.lock().unwrap().log_expire(key, seconds);
        }
    }
    value
}

pub fn ttl(key: &str) -> Option<u64> {
    let map = get_value_storage().lock().unwrap();
    match map.get(key) {
        Some(entry) => entry.expires_at,
        None => None,
    }
}

pub fn get(key: &str) -> Option<String> {
    let mut map = get_value_storage().lock().unwrap();
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
    let value_map = get_value_storage().lock().unwrap();
    let list_map = get_list_storage().lock().unwrap();
    let set_map = get_set_storage().lock().unwrap();
    value_map.contains_key(key) || list_map.contains_key(key) || set_map.contains_key(key)
}

pub fn remove_internal(key: &str) -> Option<String> {
    let mut map = get_value_storage().lock().unwrap();
    map.remove(key).map(|entry| entry.value)
}
pub fn remove(key: &str) -> Option<String> {
    let value = remove_internal(key);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_remove(key);
    }
    value
}

// MSET does not support multi-word values
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
    let mut map = get_value_storage().lock().unwrap();
    match map.get_mut(key) {
        None => Err("ERR key not found"),
        Some(entry) => match entry.value.parse::<i64>() {
            Err(_) => Err("ERR value is not an integer"),
            Ok(num) => {
                let new_value = num + increase;
                entry.value = new_value.to_string(); // muteer in-place, expires_at blijft intact
                Ok(new_value)
            }
        },
    }
}

pub fn increment(key: &str) -> Result<i64, &'static str> {
    let result = add_internal(key, 1);
    if result.is_ok() {
        if let Some(journal) = journal::JOURNAL.get() {
            journal.lock().unwrap().log_increment(&key);
        }
    }
    result
}

pub fn decrement(key: &str) -> Result<i64, &'static str> {
    let result = add_internal(key, -1);
    if result.is_ok() {
        if let Some(journal) = journal::JOURNAL.get() {
            journal.lock().unwrap().log_decrement(&key);
        }
    }
    result
}

pub fn add(key: &str, num: i64) -> Result<i64, &'static str> {
    let result = add_internal(key, num);
    if result.is_ok() {
        if let Some(journal) = journal::JOURNAL.get() {
            journal.lock().unwrap().log_add(&key, num);
        }
    }
    result
}

pub fn subtract(key: &str, num: i64) -> Result<i64, &'static str> {
    let result = add_internal(key, -num);
    if result.is_ok() {
        if let Some(journal) = journal::JOURNAL.get() {
            journal.lock().unwrap().log_subtract(&key, num);
        }
    }
    result
}

pub fn rename_internal(from: &str, to: &str) -> bool {
    let mut map = get_value_storage().lock().unwrap();
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
    if value {
        if let Some(journal) = journal::JOURNAL.get() {
            journal.lock().unwrap().log_rename(from, to);
        }
    }
    value
}

// LIST functions

pub fn list_exists(key: &str) -> bool {
    get_list_storage().lock().unwrap().contains_key(key)
}

pub fn push_right_internal(key: &str, value: &str) {
    let mut lists = get_list_storage().lock().unwrap();
    lists
        .entry(key.to_string())
        .or_insert_with(Vec::new)
        .push(value.to_string());
}
pub fn push_right(key: &str, value: &str) {
    push_right_internal(key, value);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_pushr(key, value);
    }
}

pub fn push_left_internal(key: &str, value: &str) {
    let mut lists = get_list_storage().lock().unwrap();
    lists
        .entry(key.to_string())
        .or_insert_with(Vec::new)
        .insert(0, value.to_string());
}

pub fn push_left(key: &str, value: &str) {
    push_left_internal(key, value);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_pushl(key, value);
    }
}

pub fn pop_right_internal(key: &str) -> Option<String> {
    let mut lists = get_list_storage().lock().unwrap();
    let list = lists.get_mut(key)?;
    let value = list.pop();
    if list.is_empty() {
        lists.remove(key);
    }
    value
}

pub fn pop_right(key: &str) -> Option<String> {
    let value = pop_right_internal(key);
    if value.is_some() {
        if let Some(journal) = journal::JOURNAL.get() {
            journal.lock().unwrap().log_popr(key);
        }
    }
    value
}

pub fn pop_left_internal(key: &str) -> Option<String> {
    let mut lists = get_list_storage().lock().unwrap();
    let list = lists.get_mut(key)?;
    if list.is_empty() {
        return None;
    }
    let value = list.remove(0);
    if list.is_empty() {
        lists.remove(key);
    }
    Some(value)
}

pub fn pop_left(key: &str) -> Option<String> {
    let value = pop_left_internal(key);
    if value.is_some() {
        if let Some(journal) = journal::JOURNAL.get() {
            journal.lock().unwrap().log_popl(key);
        }
    }
    value
}

pub fn llen(key: &str) -> usize {
    let lists = get_list_storage().lock().unwrap();
    lists.get(key).map_or(0, |list| list.len())
}

pub fn lrange(key: &str, start: i64, stop: i64) -> Option<Vec<String>> {
    let lists = get_list_storage().lock().unwrap();
    let list = lists.get(key)?;
    let len = list.len() as i64;

    let start = if start < 0 {
        (len + start).max(0)
    } else {
        start.min(len)
    };
    let stop = if stop < 0 {
        (len + stop + 1).max(0)
    } else {
        (stop + 1).min(len)
    };

    if start >= stop {
        return Some(vec![]);
    }

    Some(
        list[start as usize..stop as usize]
            .iter()
            .cloned()
            .collect(),
    )
}

// Set functions

pub fn set_exists(key: &str) -> bool {
    get_set_storage().lock().unwrap().contains_key(key)
}

pub fn set_add_internal(key: &str, member: &str) -> bool {
    let mut sets = get_set_storage().lock().unwrap();
    sets.entry(key.to_string())
        .or_insert_with(HashSet::new)
        .insert(member.to_string())
}

pub fn set_add(key: &str, member: &str) -> bool {
    let result = set_add_internal(key, member);
    if result {
        if let Some(journal) = journal::JOURNAL.get() {
            journal.lock().unwrap().log_sadd(key, member);
        }
    }
    result
}

pub fn set_members(key: &str) -> Option<Vec<String>> {
    let sets = get_set_storage().lock().unwrap();
    let set = sets.get(key)?;
    Some(set.iter().cloned().collect())
}

pub fn set_remove_internal(key: &str, member: &str) -> bool {
    let mut sets = get_set_storage().lock().unwrap();
    match sets.get_mut(key) {
        None => false,
        Some(set) => {
            let removed = set.remove(member);
            if set.is_empty() {
                sets.remove(key);
            }
            removed
        }
    }
}

pub fn set_remove(key: &str, member: &str) -> bool {
    let result = set_remove_internal(key, member);
    if result {
        if let Some(journal) = journal::JOURNAL.get() {
            journal.lock().unwrap().log_srem(key, member);
        }
    }
    result
}

pub fn set_is_member(key: &str, member: &str) -> bool {
    let sets = get_set_storage().lock().unwrap();
    sets.get(key).map_or(false, |set| set.contains(member))
}

pub fn set_card(key: &str) -> usize {
    let sets = get_set_storage().lock().unwrap();
    sets.get(key).map_or(0, |set| set.len())
}

pub fn set_union(key1: &str, key2: &str) -> Vec<String> {
    let sets = get_set_storage().lock().unwrap();
    let empty = HashSet::new();
    let set1 = sets.get(key1).unwrap_or(&empty);
    let set2 = sets.get(key2).unwrap_or(&empty);
    set1.union(set2).cloned().collect()
}

pub fn set_intersection(key1: &str, key2: &str) -> Vec<String> {
    let sets = get_set_storage().lock().unwrap();
    let empty = HashSet::new();
    let set1 = sets.get(key1).unwrap_or(&empty);
    let set2 = sets.get(key2).unwrap_or(&empty);
    set1.intersection(set2).cloned().collect()
}
pub fn set_difference(key1: &str, key2: &str) -> Vec<String> {
    let sets = get_set_storage().lock().unwrap();
    let empty = HashSet::new();
    let set1 = sets.get(key1).unwrap_or(&empty);
    let set2 = sets.get(key2).unwrap_or(&empty);
    set1.difference(set2).cloned().collect()
}

// hash functions
pub fn hash_set_internal(key: &str, field: &str, value: &str) {
    let mut hashes = get_hash_storage().lock().unwrap();
    hashes
        .entry(key.to_string())
        .or_insert_with(HashMap::new)
        .insert(field.to_string(), value.to_string());
}

pub fn hash_set(key: &str, field: &str, value: &str) {
    hash_set_internal(key, field, value);
    if let Some(journal) = journal::JOURNAL.get() {
        journal.lock().unwrap().log_hset(key, field, value);
    }
}

pub fn hash_get(key: &str, field: &str) -> Option<String> {
    let hashes = get_hash_storage().lock().unwrap();
    hashes.get(key)?.get(field).cloned()
}

pub fn hash_get_all(key: &str) -> Option<Vec<(String, String)>> {
    let hashes = get_hash_storage().lock().unwrap();
    let hash = hashes.get(key)?;
    Some(hash.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}

pub fn hash_delete_internal(key: &str, field: &str) -> bool {
    let mut hashes = get_hash_storage().lock().unwrap();
    match hashes.get_mut(key) {
        None => false,
        Some(hash) => {
            let removed = hash.remove(field).is_some();
            if hash.is_empty() {
                hashes.remove(key);
            }
            removed
        }
    }
}

pub fn hash_delete(key: &str, field: &str) -> bool {
    let result = hash_delete_internal(key, field);
    if result {
        if let Some(journal) = journal::JOURNAL.get() {
            journal.lock().unwrap().log_hdel(key, field);
        }
    }
    result
}

pub fn hash_exists(key: &str, field: &str) -> bool {
    let hashes = get_hash_storage().lock().unwrap();
    hashes
        .get(key)
        .map_or(false, |hash| hash.contains_key(field))
}

pub fn hash_keys(key: &str) -> Option<Vec<String>> {
    let hashes = get_hash_storage().lock().unwrap();
    Some(hashes.get(key)?.keys().cloned().collect())
}

pub fn hash_vals(key: &str) -> Option<Vec<String>> {
    let hashes = get_hash_storage().lock().unwrap();
    Some(hashes.get(key)?.values().cloned().collect())
}

pub fn hash_len(key: &str) -> usize {
    let hashes = get_hash_storage().lock().unwrap();
    hashes.get(key).map_or(0, |hash| hash.len())
}
