use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct AdminUser {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Setting {
    pub name: String,
    pub domain: String,
    pub copyright: String,
    pub license: String,
}
