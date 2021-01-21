use chrono::prelude::*;

use blog_common::{
    dto::user::{RegisterParams, UserInfo},
    result::Error,
};

use crate::{
    db::{self, DATASOURCE},
    model::User,
    result::Result,
    util::{crypt, snowflake},
};

pub async fn register(email: &str, password: &str) -> Result<UserInfo> {
    let r: Option<User> = db::sled_get(&DATASOURCE.get().unwrap().user, email).await?;
    if r.is_some() {
        return Err(Error::AlreadyRegistered.into());
    }
    let id = snowflake::gen_id() as i64;
    let user = User {
        id,
        email: email.to_owned(),
        password: crypt::encrypt_password(password),
        created_at: Utc::now().second() as u64,
    };
    match db::sled_save(&DATASOURCE.get().unwrap().user, email, &user).await {
        Ok(_) => Ok(user.into()),
        Err(e) => {
            dbg!(e);
            Err(Error::RegisterFailed.into())
        },
    }
}

pub async fn login(username: &str, password: &str) -> Result<UserInfo> {
    let r: Option<User> = db::sled_get(&DATASOURCE.get().unwrap().user, username).await?;
    if r.is_none() {
        return Err(Error::LoginFailed.into());
    }
    let u = r.unwrap();
    if crate::util::crypt::verify_password(password, &u.password) {
        Ok(u.into())
    } else {
        Err(Error::LoginFailed.into())
    }
}
