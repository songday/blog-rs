use alloc::string::String;

use lazy_static::lazy_static;
use parking_lot::RwLock;
use yew::services::storage::{Area, StorageService};

lazy_static! {
    /// Jwt token read from local storage.
    pub static ref USER_ACCESS_TOKEN: RwLock<Option<String>> = {
        RwLock::new(None)
    };
}

pub fn get(key: &str) -> Option<String> {
    USER_ACCESS_TOKEN.read();
    let storage = StorageService::new(Area::Local).expect("storage was disabled by the user");
    match storage.restore(key) {
        Ok(d) => Some(d),
        Err(e) => None,
    }
}

pub fn save(key: &str, value: Option<String>) {
    let mut storage = StorageService::new(Area::Local).expect("storage was disabled by the user");
    if let Some(t) = value.clone() {
        storage.store(key, Ok(t));
    } else {
        storage.remove(key);
    }
    let mut token_lock = USER_ACCESS_TOKEN.write();
    *token_lock = value;
}
