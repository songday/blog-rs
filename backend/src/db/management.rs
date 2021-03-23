use blog_common::result::Error;
use chrono::prelude::*;

use crate::{
    db::{self, model::User, DATA_SOURCE},
    util::{crypt, result::Result},
};

async fn get_admin_user() -> Option<User> {
    match db::sled_get(&DATA_SOURCE.get().unwrap().setting, "admin_user").await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e.0);
            None
        },
    }
}

pub async fn have_admin() -> bool { get_admin_user().await.is_some() }

pub async fn update_admin(email: &str, password: &str) -> Result<()> {
    let admin = User {
        id: 1,
        email: email.to_owned(),
        password: crypt::encrypt_password(password)?,
        created_at: Utc::now().second() as i64,
    };
    let _r = db::sled_save(&DATA_SOURCE.get().unwrap().setting, "admin_user", &admin).await?;
    Ok(())
}

pub async fn admin_register(email: &str, password: &str) -> Result<()> {
    let u = get_admin_user().await;
    if u.is_some() {
        return Err(Error::BusinessException("已有管理用户，若忘记密码，请使用“找回密码”功能".into()).into());
    }
    update_admin(email, password).await
}

pub async fn admin_login(email: &str, password: &str) -> Result<()> {
    println!("check admin user");
    let u = get_admin_user().await;
    println!("check admin user 1");
    if u.is_none() {
        return Err(Error::LoginFailed.into());
    }
    println!("check admin user 2");
    let u = u.unwrap();
    if u.email.eq(email) && crypt::verify_password(password, &u.password)? {
        println!("check admin user 3");
        return Ok(());
    }
    println!("check admin user 4");
    Err(Error::LoginFailed.into())
}
