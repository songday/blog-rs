use blog_common::{
    dto::{management::Settings, user::UserInfo},
    result::Error,
};
use chrono::prelude::*;
use sqlx::{Row, Sqlite};

use crate::{
    db::{self, model::User, DATA_SOURCE},
    service::status,
    util::{crypt, result::Result},
};

pub async fn has_settings() -> Result<bool> {
    let row = sqlx::query("SELECT COUNT(id) FROM settings WHERE admin_password != ''")
        .fetch_one(db::get_sqlite())
        .await?;
    let total: i64 = row.get(0);
    return Ok(total > 0)
}

pub async fn admin_login(token: &str, password: &str) -> Result<bool> {
    let d = sqlx::query_as::<Sqlite, Settings>(
        "SELECT admin_password,'' AS name,'' AS domain,'' AS copyright,'' AS license FROM post ORDER BY id DESC LIMIT 1",
    )
        .fetch_one(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    if crypt::verify_password(password, &d.admin_password)? {
        let u: UserInfo = (&u).into();
        status::user_online(token, u.clone());
        return Ok(true);
    }
    return Ok(false)
}

pub async fn settings() -> Result<Settings> {
    let settings: Option<Settings> = db::sled_get(&DATA_SOURCE.get().unwrap().management, "settings").await?;
    Ok(settings.unwrap_or(Settings::default()))
}

pub async fn update_settings(token: Option<String>, settings: Settings) -> Result<()> {
    status::check_auth(token)?;
    // db::sled_save(&DATA_SOURCE.get().unwrap().management, "settings", &setting).await?;
    let sql;
    if settings.admin_password.is_empty() {
        sql = sqlx::query("UPDATE settings SET name=?,domain=?,copyright=?,license=? WHERE id=1")
            .bind(&settings.name)
            .bind(&settings.domain)
            .bind(&settings.copyright)
            .bind(&settings.license);
    } else {
        sql = sqlx::query("UPDATE settings SET name=?,domain=?,copyright=?,license=?,admin_password=? WHERE id=1")
            .bind(&settings.name)
            .bind(&settings.domain)
            .bind(&settings.copyright)
            .bind(&settings.license)
            .bind(&settings.admin_password);
    }

    sql.execute(db::get_sqlite()).await?;
    Ok(())
}
