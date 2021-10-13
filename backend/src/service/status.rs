use std::{collections::HashMap, sync::Arc, vec::Vec};

use lazy_static::lazy_static;
use parking_lot::RwLock;
use tokio::time::{sleep, Duration};

use blog_common::{dto::user::UserInfo, result::Error, util::time};

use crate::util::result::Result;

const MAX_USER_IDLE_MILLIS: u64 = 1800000;
const MAX_VERIFY_CODE_IDLE_MILLIS: u64 = 300000;

type OnlineUsers = HashMap<String, OnlineUser>;
type VerifyCodes = HashMap<String, VerifyCode>;

lazy_static! {
    static ref ONLINE_USERS: Arc<RwLock<OnlineUsers>> = Arc::new(RwLock::new(HashMap::with_capacity(32)));
    static ref VERIFY_CODES: Arc<RwLock<VerifyCodes>> = Arc::new(RwLock::new(HashMap::with_capacity(128)));
}

struct OnlineUser {
    user: UserInfo,
    // #[serde(skip)]
    // https://serde.rs/field-attrs.html
    pub last_active_time: u64,
}

struct VerifyCode {
    code: Vec<u8>,
    // #[serde(skip)]
    // https://serde.rs/field-attrs.html
    pub last_active_time: u64,
}

pub async fn scanner() {
    let mut current_timestamp = 0u64;
    loop {
        // println!("Scanning online users and verify codes");
        current_timestamp = time::current_timestamp();
        {
            let mut online_users = ONLINE_USERS.write();
            let d = &mut *online_users;
            d.retain(|_, v| {
                if current_timestamp - v.last_active_time > MAX_USER_IDLE_MILLIS {
                    println!("Remove user {}", dbg!(&v.user.id));
                    false
                } else {
                    true
                }
            });
        }
        {
            let mut verify_codes = VERIFY_CODES.write();
            let d = &mut *verify_codes;
            d.retain(|k, v| {
                if current_timestamp - v.last_active_time > MAX_VERIFY_CODE_IDLE_MILLIS {
                    println!("Remove verifyCode {}", dbg!(k));
                    false
                } else {
                    true
                }
            });
        }
        sleep(Duration::from_secs(10)).await;
    }
}

pub(crate) fn check_auth(token: Option<String>) -> Result<UserInfo> {
    if token.is_none() {
        return Err(Error::NotAuthed.into());
    }
    let token = token.unwrap();
    if token.len() != 32 {
        return Err(Error::NotAuthed.into());
    }
    let mut r = ONLINE_USERS.write();
    let d = &mut *r;
    if let Some(u) = d.get_mut(&token) {
        u.last_active_time = time::current_timestamp();
        return Ok(u.user.clone());
    }
    Err(Error::NotAuthed.into())
}

pub(crate) fn user_online(token: &str, user: UserInfo) {
    ONLINE_USERS.write().insert(
        String::from(token),
        OnlineUser {
            user: user.clone(),
            last_active_time: time::current_timestamp(),
        },
    );
}

pub(crate) fn user_offline(token: &str) { ONLINE_USERS.write().remove(token); }

pub fn get_verify_code(token: &str) -> Result<Vec<u8>> {
    if token.len() != 32 {
        return Err(Error::InvalidVerifyCode.into());
    }
    {
        let r = VERIFY_CODES.read();
        if let Some(v) = r.get(token) {
            return Ok(v.code.clone());
        }
    }
    let numbers: Vec<u8> = crate::util::num::rand_numbers(0, 10, 4);
    VERIFY_CODES.write().insert(
        String::from(token),
        VerifyCode {
            code: numbers.clone(),
            last_active_time: time::current_timestamp(),
        },
    );
    Ok(numbers)
}

// pub fn check_verify_code(token: &str, code: &str) -> bool {
pub fn check_verify_code(token: Option<String>, code: &str) -> Result<String> {
    if token.is_none() {
        return Err(Error::InvalidSessionId.into());
    }
    let token = token.unwrap();
    if token.len() != 32 {
        return Err(Error::InvalidSessionId.into());
    }
    let valid_code = {
        let r = VERIFY_CODES.read();
        if let Some(v) = r.get(&token) {
            let mut s = String::with_capacity(8);
            for c in v.code.iter() {
                s.push_str(c.to_string().as_str());
            }
            s.as_str() == code
        } else {
            false
        }
    };
    if valid_code {
        VERIFY_CODES.write().remove(&token);
    }
    Ok(token)
}
