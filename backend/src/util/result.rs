use blog_common::result::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub type Result<D> = core::result::Result<D, ErrorWrapper>;
// pub type Result<D> = core::result::Result<D, Error>;

#[derive(Debug)]
pub struct ErrorWrapper(pub(crate) Error);

impl From<Error> for ErrorWrapper {
    fn from(e: Error) -> Self {
        ErrorWrapper(e)
    }
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

// impl std::error::Error for ErrResponse {}

impl warp::reject::Reject for ErrorWrapper {}

impl From<std::io::Error> for ErrorWrapper {
    fn from(e: std::io::Error) -> Self {
        eprintln!("{}", e);
        Error::ReadPostIdDataByTagFailed.into()
    }
}

impl From<reqwest::Error> for ErrorWrapper {
    fn from(e: reqwest::Error) -> Self {
        eprintln!("{}", e);
        Error::ReadPostIdDataByTagFailed.into()
    }
}

impl From<std::time::SystemTimeError> for ErrorWrapper {
    fn from(e: std::time::SystemTimeError) -> Self {
        eprintln!("{}", e);
        Error::ReadPostIdDataByTagFailed.into()
    }
}

// impl From<std::env::VarError> for ErrorWrapper {
//     fn from(e: std::env::VarError) -> Self {
//         eprintln!("{}", e);
//         Error::EnvVarError.into()
//     }
// }

impl From<std::net::AddrParseError> for ErrorWrapper {
    fn from(e: std::net::AddrParseError) -> Self {
        eprintln!("{}", e);
        Error::ParseListeningAddressFailed.into()
    }
}

impl From<serde_json::error::Error> for ErrorWrapper {
    fn from(e: serde_json::error::Error) -> Self {
        eprintln!("{}", e);
        ErrorWrapper(Error::SerdeError)
    }
}

impl From<sled::Error> for ErrorWrapper {
    fn from(e: sled::Error) -> Self {
        eprintln!("{}", e);
        ErrorWrapper(Error::SledDbError)
    }
}

impl From<sqlx::Error> for ErrorWrapper {
    fn from(e: sqlx::Error) -> Self {
        eprintln!("{}", dbg!(e));
        ErrorWrapper(Error::SqliteDbError)
    }
}

// impl ErrResponse {
//     pub fn new(message: &str) -> Self {
//         ErrResponse {
//             message: String::from(message)
//         }
//     }
// }

impl From<std::string::FromUtf8Error> for ErrorWrapper {
    fn from(e: std::string::FromUtf8Error) -> Self {
        eprintln!("{:?}", e);
        ErrorWrapper(Error::BadRequest)
    }
}

impl From<argon2::Error> for ErrorWrapper {
    fn from(e: argon2::Error) -> Self {
        eprintln!("{:?}", e);
        ErrorWrapper(Error::BadRequest)
    }
}

impl From<base64::DecodeError> for ErrorWrapper {
    fn from(e: base64::DecodeError) -> Self {
        eprintln!("{:?}", e);
        ErrorWrapper(Error::BadRequest)
    }
}

// impl From<scrypt::errors::InvalidOutputLen> for ErrorWrapper {
//     fn from(e: scrypt::errors::InvalidOutputLen) -> Self {
//         eprintln!("{:?}", e);
//         ErrorWrapper(Error::BadRequest)
//     }
// }
