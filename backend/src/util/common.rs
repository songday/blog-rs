use lazy_static::lazy_static;
use rand::{rngs::OsRng, RngCore};
use regex::Regex;
use uuid::Uuid;

pub fn simple_uuid_with_name(name: &[u8]) -> String {
    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, name);
    uuid.to_simple().to_string()
}

pub fn simple_uuid() -> String {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    simple_uuid_with_name(&salt)
}

lazy_static! {
    pub static ref EMAIL_REGEX: Regex = Regex::new(r"[^@ \t\r\n]+@[^@ \t\r\n]+\.[^@ \t\r\n]+").unwrap();
}
