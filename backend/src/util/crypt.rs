use base64;
use rand::{rngs::OsRng, RngCore};
use scrypt::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Scrypt,
};
// use subtle::ConstantTimeEq;

pub fn encrypt_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    encrypt_password_with_salt(password.as_bytes(), &salt)
}

fn encrypt_password_with_salt(password: &[u8], salt: &SaltString) -> String {
    Scrypt
        .hash_password_simple(password, salt.as_ref())
        .unwrap()
        .to_string()
}

pub fn verify_password(password: &str, encrypted_password: &str) -> bool {
    let parsed_hash = PasswordHash::new(&encrypted_password).unwrap();
    Scrypt.verify_password(password.as_bytes(), &parsed_hash).is_ok()
}
