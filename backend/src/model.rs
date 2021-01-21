use core::fmt::Display;

use chrono::{prelude::*, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use blog_common::dto::{blog::BlogDetail, user::UserInfo};
use sqlx::{
    database::{HasArguments, HasValueRef},
    encode::IsNull,
    error::BoxDynError,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub created_at: u64,
}

impl Into<UserInfo> for User {
    fn into(self) -> UserInfo {
        UserInfo {
            id: self.id,
            email: self.email,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Blog {
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
    pub parsed_content: String,
    pub tags: String,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

impl Into<BlogDetail> for Blog {
    fn into(self) -> BlogDetail {
        let tags = if self.tags.is_empty() {
            None
        } else {
            Some(self.tags.split(|c| c == '\n').map(|s| String::from(s)).collect())
        };
        BlogDetail {
            id: self.id,
            title: self.title,
            content: self.parsed_content,
            tags,
            created_at: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.created_at, 0), Utc),
            updated_at: None,
        }
    }
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct Tag {
    pub name: String,
}
