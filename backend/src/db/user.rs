use chrono::prelude::*;
use sqlx::Sqlite;

use blog_common::{dto::user::UserInfo, result::Error};

use crate::{
    db::{self, model::User, DATA_SOURCE},
    util::{crypt, result::Result, snowflake},
};

pub async fn register(email: &str, password: &str) -> Result<UserInfo> {
    let r = sqlx::query("SELECT id FROM user WHERE email = ?")
        .bind(email)
        .fetch_optional(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    if r.is_some() {
        return Err(Error::AlreadyRegistered.into());
    }

    let user = User {
        id: snowflake::gen_id() as i64,
        email: email.to_owned(),
        password: crypt::encrypt_password(password)?,
        created_at: Utc::now().second() as i64,
    };

    let r = sqlx::query("INSERT INTO user(id,email,password,created_at) VALUES(?,?,?,?)")
        .bind(&user.id)
        .bind(email)
        .bind(&user.password)
        .bind(user.created_at as i64)
        .execute(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    if r.rows_affected() < 1 {
        return Err(Error::RegisterFailed.into());
    }

    Ok((&user).into())
}

pub async fn login(email: &str, password: &str) -> Result<UserInfo> {
    let r = sqlx::query_as::<Sqlite, User>("SELECT * FROM user WHERE email = ?")
        .bind(email)
        .fetch_optional(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    if r.is_none() {
        return Err(Error::LoginFailed.into());
    }

    let u = r.unwrap();
    if crate::util::crypt::verify_password(password, &u.password)? {
        Ok((&u).into())
    } else {
        Err(Error::LoginFailed.into())
    }
}
