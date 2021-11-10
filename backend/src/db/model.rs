use core::fmt::Display;

use serde::{Deserialize, Serialize};

use blog_common::dto::{post::PostDetail, user::UserInfo};
use sqlx::{
    database::{HasArguments, HasValueRef},
    encode::IsNull,
    error::BoxDynError,
};

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub created_at: i64,
}

impl Into<UserInfo> for &User {
    fn into(self) -> UserInfo {
        UserInfo { id: self.id }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Post {
    /*
    https://docs.rs/sqlx/0.4.0-beta.1/sqlx/prelude/trait.Type.html

    这里想自己实现`id`的`u64`，因为`sqlx`只实现了`i64`，然后根据文档，自己实现`Encode`和`Encode`
    https://docs.rs/sqlx/0.4.0-beta.1/sqlx/prelude/trait.Encode.html#impl-Encode%3C%27q%2C%20Sqlite%3E-for-i64
    https://docs.rs/sqlx/0.4.0-beta.1/sqlx/prelude/trait.Decode.html?search=#impl-Decode%3C%27r%2C%20Sqlite%3E-for-i64
    但是实现`Decode`的方法的时候，里面的对象都是私有的，走进死胡同了
    后来想了下，为什么只有`MySQL`实现了`u64`，原因是只有`MySQL`支持`unsigned bigint`类型的字段

    参考了其它：
    https://docs.rs/sqlx/0.4.0-beta.1/sqlx/trait.TypeInfo.html
    https://docs.rs/sqlx/0.4.0-beta.1/sqlx/sqlite/struct.SqliteTypeInfo.html
    https://docs.rs/sqlx/0.4.0-beta.1/sqlx/prelude/trait.Type.html#impl-Type%3CSqlite%3E-for-i64
    https://docs.rs/sqlx/0.4.0-beta.1/sqlx/struct.Pool.html
     */
    pub id: i64,
    pub title: String,
    pub markdown_content: String,
    pub rendered_content: String,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

impl Into<PostDetail> for &Post {
    fn into(self) -> PostDetail {
        PostDetail {
            id: self.id,
            title: self.title.clone(),
            content: self.rendered_content.clone(),
            tags: None,
            created_at: self.created_at as u64,
            updated_at: self.updated_at.map(|t| t as u64),
            editable: false,
        }
    }
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct TagUsage {
    pub id: i64,
    pub post_id: i64,
    pub tag_id: i64,
}

#[derive(Clone, Default, Debug, Serialize, sqlx::FromRow)]
pub struct Settings {
    pub admin_password: String,
    pub name: String,
    pub domain: String,
    pub copyright: String,
    pub license: String,
    // pub settings: blog_common::dto::management::Settings,
}

// impl std::ops::Deref for Settings {
//     type Target = blog_common::dto::management::Settings;
//     fn deref(&self) -> &Self::Target {
//         &self.settings
//     }
// }

impl From<blog_common::dto::management::Settings> for Settings {
    fn from(settings: blog_common::dto::management::Settings) -> Self {
        let admin_password = settings.admin_password;
        let name = settings.name;
        let domain = settings.domain;
        let copyright = settings.copyright;
        let license = settings.license;
        Self {
            admin_password,
            name,
            domain,
            copyright,
            license,
        }
    }
}
