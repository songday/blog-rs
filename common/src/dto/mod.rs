use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::result::ErrorResponse;

pub mod post;
pub mod user;
mod setting;

//https://stackoverflow.com/questions/49953960/cannot-resolve-t-serdedeserializea-when-deriving-deserialize-on-a-generic
//https://stackoverflow.com/questions/54761790/how-to-deserialize-with-for-a-container-using-serde-in-rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Response<D> {
    pub status: u16,
    pub error: Option<ErrorResponse>,
    #[serde(bound(deserialize = "D: Deserialize<'de>", serialize = "D: Serialize"))]
    pub data: Option<D>,
}

pub struct UploadFileInfo {
    pub origin_filename: String,
    pub filepath: PathBuf,
    pub new_filename_len: usize,
    pub filesize: usize,
    pub extension: String,
}

impl UploadFileInfo {
    pub fn new() -> Self {
        UploadFileInfo {
            origin_filename: String::with_capacity(128),
            filepath: PathBuf::with_capacity(128),
            new_filename_len: 0,
            filesize: 0,
            extension: String::with_capacity(16),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaginationData<D> {
    pub total: u64,
    #[serde(bound(deserialize = "D: Deserialize<'de>", serialize = "D: Serialize"))]
    pub data: D,
}
