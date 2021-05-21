use blog_common::{
    dto::{management::Setting, user::UserInfo},
    result::Error,
};
use chrono::prelude::*;

use crate::{
    db::{self, model::User, DATA_SOURCE},
    service::status,
    util::{crypt, result::Result},
};

async fn get_admin_user() -> Option<User> {
    match db::sled_get(&DATA_SOURCE.get().unwrap().management, "admin_user").await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e.0);
            None
        },
    }
}

pub async fn have_admin() -> bool { get_admin_user().await.is_some() }

pub async fn update_admin(email: &str, password: &str) -> Result<User> {
    let admin = User {
        id: 1,
        email: email.to_owned(),
        password: crypt::encrypt_password(password)?,
        created_at: Utc::now().second() as i64,
    };
    let _r = db::sled_save(&DATA_SOURCE.get().unwrap().management, "admin_user", &admin).await?;
    Ok(admin)
}

pub async fn admin_register(email: &str, password: &str) -> Result<UserInfo> {
    let u = get_admin_user().await;
    if u.is_some() {
        return Err(Error::BusinessException("已有管理用户，若忘记密码，请使用“找回密码”功能".into()).into());
    }
    let u = update_admin(email, password).await?;
    Ok((&u).into())
}

pub async fn admin_login(token: &str, email: &str, password: &str) -> Result<UserInfo> {
    let u = get_admin_user().await;
    if u.is_none() {
        return Err(Error::LoginFailed.into());
    }
    let u = u.unwrap();
    if u.email.eq(email) && crypt::verify_password(password, &u.password)? {
        let u: UserInfo = (&u).into();
        status::user_online(token, u.clone());
        return Ok(u);
    }
    Err(Error::LoginFailed.into())
}

pub async fn settings() -> Result<Setting> {
    let settings: Option<Setting> = db::sled_get(&DATA_SOURCE.get().unwrap().management, "settings").await?;
    Ok(settings.unwrap_or(Setting::default()))
}

pub async fn update_settings(token: Option<String>, setting: Setting) -> Result<()> {
    status::check_auth(token)?;
    db::sled_save(&DATA_SOURCE.get().unwrap().management, "settings", &setting).await?;
    Ok(())
}
