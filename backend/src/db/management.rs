use blog_common::{
    dto::{user::UserInfo},
    result::Error,
};
use chrono::prelude::*;
use sqlx::{Row, Sqlite};

use crate::{
    db::{self, model::{User, Settings}, DATA_SOURCE},
    service::status,
    util::{crypt, result::Result},
};

pub async fn has_settings() -> Result<bool> {
    let row = sqlx::query("SELECT COUNT(id) FROM settings WHERE id=1 AND admin_password != ''")
        .fetch_one(db::get_sqlite())
        .await?;
    let total: i64 = row.get(0);
    dbg!(total);
    return Ok(total > 0)
}

pub async fn admin_login(token: &str, password: &str) -> Result<bool> {
    let d = sqlx::query_as::<Sqlite, crate::db::model::Settings>(
        "SELECT admin_password,'' AS name,'' AS domain,'' AS copyright,'' AS license FROM settings WHERE id=1",
    ).fetch_optional(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;

    if let Some(settings) = d {
        if crypt::verify_password(password, &settings.admin_password)? {
            status::user_online(token, UserInfo {
                id: 1,
                email: "".to_owned(),
            });
            return Ok(true);
        }
    }
    return Ok(false)
}

pub async fn settings() -> Result<Settings> {
    // let settings: Option<Settings> = db::sled_get(&DATA_SOURCE.get().unwrap().management, "settings").await?;
    let settings = sqlx::query_as::<Sqlite, crate::db::model::Settings>("SELECT * FROM settings WHERE id=1").fetch_optional(super::get_sqlite()).await?;
    Ok(settings.unwrap_or(Settings::default()))
}

pub async fn update_settings(token: Option<String>, settings: Settings) -> Result<()> {
    status::check_auth(token)?;
    // db::sled_save(&DATA_SOURCE.get().unwrap().management, "settings", &setting).await?;
    let encrypted_password = if settings.admin_password.is_empty() {
        String::new()
    } else {
        crypt::encrypt_password(&settings.admin_password)?
    };

    let r = sqlx::query("UPDATE settings SET name=?,domain=?,copyright=?,license=?,admin_password=? WHERE id=1")
        .bind(&settings.name)
        .bind(&settings.domain)
        .bind(&settings.copyright)
        .bind(&settings.license)
        .bind(&encrypted_password).execute(db::get_sqlite()).await?;

    if r.rows_affected() < 1 {
        sqlx::query("INSERT INTO settings(id,name,domain,copyright,license,admin_password)VALUES(1,?,?,?,?,?)")
            .bind(&settings.name)
            .bind(&settings.domain)
            .bind(&settings.copyright)
            .bind(&settings.license)
            .bind(&encrypted_password).execute(db::get_sqlite()).await?;
    }
    Ok(())
}
