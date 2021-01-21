use base64;
use rand::{rngs::OsRng, RngCore};
use scrypt::{scrypt, ScryptParams};
use subtle::ConstantTimeEq;

fn get_scrypt_params() -> ScryptParams { ScryptParams::new(6, 8, 1).unwrap() }

pub fn encrypt_password(password: &str) -> String {
    // code from scrypt::scrypt_simple
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    encrypt_password_with_salt(password, &salt)
}

fn encrypt_password_with_salt(password: &str, salt: &[u8]) -> String {
    let mut dk = [0u8; 32];

    scrypt(password.as_bytes(), &salt, &get_scrypt_params(), &mut dk)
        .expect("32 bytes always satisfy output length requirements");

    let mut result = String::with_capacity(128);
    result.push_str(&base64::encode(&salt));
    result.push('$');
    result.push_str(&base64::encode(&dk));

    result
}

pub fn verify_password(password: &str, encrypted_password: &str) -> bool {
    // code from scrypt::scrypt_check
    let iter = encrypted_password.split('$');
    // for s in iter {
    //     println!("{}", s)
    // }
    let vec = iter.collect::<Vec<&str>>();
    // OR let vec: Vec<&str> = iter.collect();
    let salt = match base64::decode(vec[0]) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let compared_password = match base64::decode(vec[1]) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let mut output = vec![0u8; compared_password.len()];
    scrypt(password.as_bytes(), &salt, &get_scrypt_params(), &mut output)
        .expect("32 bytes always satisfy output length requirements");

    output.ct_eq(&compared_password).unwrap_u8() == 1
}
