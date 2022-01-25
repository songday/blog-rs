use serde::{Deserialize, Serialize};

use super::user::UserInfo;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct AdminUser {
    pub password: String,
    pub captcha: String,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub admin_password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SiteData {
    pub settings: Settings,
    pub user_info: Option<UserInfo>,
}
