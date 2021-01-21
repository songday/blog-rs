use rand::{rngs::OsRng, RngCore};
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
