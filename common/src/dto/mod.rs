use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::result::ErrorResponse;

pub mod git;
pub mod management;
pub mod post;
pub mod tag;
pub mod user;

//https://stackoverflow.com/questions/49953960/cannot-resolve-t-serdedeserializea-when-deriving-deserialize-on-a-generic
//https://stackoverflow.com/questions/54761790/how-to-deserialize-with-for-a-container-using-serde-in-rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Response<D> {
    pub status: u16,
    pub error: Option<ErrorResponse>,
    #[serde(bound(deserialize = "D: Deserialize<'de>", serialize = "D: Serialize"))]
    pub data: Option<D>,
}

pub enum FormDataItem {
    TEXT(TextFieldInfo),
    FILE(UploadFileInfo),
}

pub struct TextFieldInfo {
    pub name: String,
    pub value: String,
}

pub struct UploadFileInfo {
    pub name: String,
    pub original_filename: String,
    pub relative_path: String,
    pub filepath: PathBuf,
    pub extension: String,
    pub filesize: usize,
}

impl UploadFileInfo {
    pub fn new() -> Self {
        UploadFileInfo {
            name: String::with_capacity(64),
            original_filename: String::with_capacity(128),
            relative_path: String::with_capacity(64),
            filepath: PathBuf::with_capacity(64),
            extension: String::with_capacity(16),
            filesize: 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaginationData<D> {
    pub total: u64,
    #[serde(bound(deserialize = "D: Deserialize<'de>", serialize = "D: Serialize"))]
    pub data: D,
}
