use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub email: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserInfoWrapper {
    pub user_info: UserInfo,
    // 由于 cookie 设置了 HttpOnly，所以 JS 读不了 cookie。就通过这个字段将 cookie 传递给前端
    pub access_token: String,
}

// #[derive(Clone, Default, Debug, Deserialize, Serialize)]
// pub struct RegisterParams {
//     pub email: String,
//     pub password1: String,
//     pub password2: String,
//     pub captcha: String,
// }
//
// #[derive(Clone, Default, Debug, Deserialize, Serialize)]
// pub struct LoginParams {
//     pub email: String,
//     pub password: String,
//     pub captcha: String,
// }

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct UserParams {
    pub email: String,
    pub password1: String,
    pub password2: String,
    pub captcha: String,
}
