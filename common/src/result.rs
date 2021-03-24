use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;
// pub type AsyncResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, ThisError, Debug, Deserialize, Serialize)]
pub enum Error {
    // system
    // EnvVarError,
    #[error("Parsing listening address failed")]
    ParseListeningAddressFailed,
    #[error("Data save failed")]
    SledSaveFailed,
    #[error("Database(1) error")]
    SledDbError,
    #[error("Database(2) error")]
    SqliteDbError,
    #[error("Deserialize / Serialize failed")]
    SerdeError,
    #[error("Page not found")]
    NotFound,
    #[error("请求参数不合法，请检查输入是否正确")]
    BadRequest,
    #[error("Method not allowed")]
    MethodNotAllowed,
    #[error("Internal server error")]
    InternalServerError,

    // business
    #[error("无效的 Session ID")]
    InvalidSessionId,
    #[error("无效的验证码")]
    InvalidVerifyCode,
    #[error("登录信息失效，请重新登录")]
    NotAuthed,
    #[error("登录失败，请重试。")]
    LoginFailed,
    #[error("Registration failed")]
    RegisterFailed,
    #[error("Already registered")]
    AlreadyRegistered,
    #[error("Saving blog failed")]
    SaveBlogFailed,
    #[error("Can not find blog you requested")]
    CannotFoundBlog,
    #[error("Can not find tag you requested")]
    CannotFoundTag,
    #[error("Upload failed")]
    UploadFailed,
    #[error("Unknown file type")]
    UnknownFileType,
    #[error("Unsupported file type {0}")]
    UnsupportedFileType(String),
    #[error("Creating thumbnail failed")]
    CreateThumbnailFailed,
    #[error("Reading blog id data by tag failed")]
    ReadBlogIdDataByTagFailed,
    #[error("Saving blog id data by tag failed")]
    SaveBlogIdDataByTagFailed,

    #[error("{0}")]
    BusinessException(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub code: Error,
    pub detail: String,
}

// 如果要在Yew前端展示，这里可以不用手动序列化，让Yew反序列化再展示出来就可以了
// impl Serialize for Error {
//     fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
//         where
//             S: Serializer,
//     {
//         format!("{}", self).serialize(serializer)
//     }
// }

// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result<String> {
//         unimplemented!()
//     }
// }

// impl std::fmt::Display for ErrResponse {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         unimplemented!()
//     }
// }
