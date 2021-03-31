use std::vec::Vec;

// https://medium.com/analytics-vidhya/password-hashing-pbkdf2-scrypt-bcrypt-and-argon2-e25aaf41598e
use argon2::Argon2;
use base64;
use rand::{rngs::OsRng, RngCore};
// use subtle::ConstantTimeEq;

use crate::util::result::Result;

// https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md#phc-string-format
fn to_phc_string(salt: &[u8], encrypted_password: &[u8]) -> String {
    // $argon2i$v=19$m=512,t=4,p=2$eM+ZMyYkpDRGaI3xXmuNcQ$c5DeJg3eb5dskVt1mDdxfw
    format!(
        "$argon2id${}${}",
        base64::encode(salt),
        base64::encode(encrypted_password)
    )
}

pub fn encrypt_password(password: &str) -> Result<String> {
    let mut salt = [0u8; 64];
    OsRng.fill_bytes(&mut salt);
    encrypt_password_salt(&salt, password.as_bytes())
}

fn encrypt_password_salt(salt: &[u8], password: &[u8]) -> Result<String> {
    let a = Argon2::new(None, 2, 10240, 2, argon2::Version::V0x13)?;
    let mut result = vec![0u8; 1024];
    a.hash_password_into(argon2::Algorithm::Argon2id, password, &salt, &[], &mut result)?;

    let p = to_phc_string(&salt, result.as_slice());
    // println!("p = {}", &p);
    Ok(p)
}

pub fn verify_password(password: &str, encrypted_password: &str) -> Result<bool> {
    let d: Vec<_> = encrypted_password.split('$').collect();
    if d.len() != 4 {
        return Ok(false);
    }
    let salt = base64::decode(*d.get(2).unwrap())?;
    let p = encrypt_password_salt(salt.as_slice(), password.as_bytes())?;
    Ok(p.eq(encrypted_password))
}
