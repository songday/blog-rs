use serde::{Deserialize, Serialize};

use super::user::UserInfo;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct AdminUser {
    pub email: String,
    pub password: String,
    pub captcha: String,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Setting {
    pub name: String,
    pub domain: String,
    pub copyright: String,
    pub license: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SiteData {
    pub settings: Setting,
    pub user_info: Option<UserInfo>,
}
