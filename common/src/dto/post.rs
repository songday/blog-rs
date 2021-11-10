// use std::{
//     fmt::{self, Display},
//     path::Path,
//     str::FromStr,
// };

use serde::{Deserialize, Serialize};

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
    pub created_at: u64,
    pub updated_at: Option<u64>,
    pub editable: bool,
}

impl PostDetail {
    pub fn default() -> Self {
        PostDetail {
            id: 0,
            title: String::new(),
            content: String::new(),
            tags: None,
            created_at: 0,
            updated_at: None,
            editable: false,
        }
    }
}

// #[allow(deadcode)]
// #[derive(Clone)]
// pub struct OptionBlogDetail(pub Option<BlogDetail>);
//
// impl Display for OptionBlogDetail {
//     fn fmt(&self, f: &mut serde_json::ser::Formatter<'_>) -> fmt::Result {
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
    pub relative_path: String,
    pub original_filename: String,
}

impl UploadImage {
    pub fn new(path: String, original_filename: String) -> Self {
        UploadImage {
            relative_path: path,
            original_filename,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub name: String,
}
