// use std::{
//     fmt::{self, Display},
//     path::Path,
//     str::FromStr,
// };

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::ser::Formatter;

// use crate::result::Error;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct PostData {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PostDetail {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub editable: bool,
}

// #[allow(deadcode)]
// #[derive(Clone)]
// pub struct OptionBlogDetail(pub Option<BlogDetail>);
//
// impl Display for OptionBlogDetail {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         if self.0.is_none() {
//             f.write_str("")
//         } else {
//             match serde_json::to_string(self.0.as_ref().unwrap()) {
//                 Ok(s) => write!(f, "{}", &s),
//                 Err(e) => f.write_str(""),
//             }
//         }
//     }
// }
//
// impl FromStr for OptionBlogDetail {
//     type Err = Error;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         if s.is_empty() {
//             return Ok(OptionBlogDetail(None));
//         }
//         unimplemented!()
//     }
// }

#[derive(Debug, Deserialize, Serialize)]
pub struct UploadImage {
    pub path: String,
    pub original_filename: String,
}

impl UploadImage {
    pub fn new(path: String, original_filename: String) -> Self {
        UploadImage {
            path,
            original_filename,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub name: String,
}
