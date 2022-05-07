use serde::{Deserialize, Serialize};

// use super::user::UserInfo;

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct AdminUser {
    pub password: String,
    pub captcha: String,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Setting {
    pub item: String,
    pub content: String,
}

// #[derive(Debug, Deserialize, Serialize)]
// pub struct SiteData {
//     pub settings: Setting,
//     pub user_info: Option<UserInfo>,
// }
