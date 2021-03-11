use blog_common::{dto::user::UserInfo, result::Error};

use crate::{
    db::{DataSource, SqlParam, user},
    service::status,
};
use crate::util::result::{ErrorWrapper, Result};

pub async fn register(token: &str, email: &str, password: &str) -> Result<UserInfo> {
    if email.len() < 6 || password.len() < 5 {
        return Err(ErrorWrapper(Error::RegisterFailed));
    }
    let u = user::register(email, password).await?;
    status::user_online(&token, u.clone());
    Ok(u)
}

pub async fn login(token: &str, email: &str, password: &str) -> Result<UserInfo> {
    if email.len() < 6 || password.len() < 5 {
        return Err(ErrorWrapper(Error::LoginFailed));
    }
    let u = user::login(email, password).await?;
    status::user_online(&token, u.clone());
    Ok(u)
}
