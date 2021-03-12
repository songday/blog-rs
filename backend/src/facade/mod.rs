pub(crate) mod user;
pub(crate) mod post;
pub(crate) mod image;
pub(crate) mod tag;
pub(crate) mod asset;

use core::{convert::Infallible, result::Result};

use bytes::Buf;
use hyper::header::{self, HeaderMap, HeaderValue};
use serde::Serialize;
use warp::{
    filters::multipart::FormData,
    http::{response::Response, StatusCode},
    reply::{Json, Response as WarpResponse},
    Rejection, Reply,
};

use blog_common::{
    dto::{
        post::NewPost,
        user::{LoginParams, RegisterParams, UserInfo, UserInfoWrapper},
        Response as ApiResponse,
    },
    result::{Error, ErrorResponse},
    val,
};

// lazy_static_include_str!(INDEX_PAGE_BYTES, "./src/asset/index.html");

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    dbg!(&err);

    let code;
    let error;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        error = Error::NotFound;
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        error = Error::BadRequest;
    } else if let Some(e) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        error = Error::MethodNotAllowed;
    } else {
        eprintln!("unhandled error: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        error = Error::InternalServerError;
    }

    let json = response_err(code.as_u16(), error);

    Ok(warp::reply::with_status(json, code))
}

#[inline]
fn wrap_data<D: Serialize>(data: D) -> ApiResponse<D> {
    ApiResponse::<D> {
        status: 0,
        error: None,
        data: Some(data),
    }
}

#[inline]
fn wrap_err(status: u16, error: Error) -> ApiResponse<String> {
    ApiResponse::<String> {
        status,
        error: Some(ErrorResponse {
            detail: format!("{}", error),
            code: error,
        }),
        data: None,
    }
}

fn response_data<D: Serialize>(data: D) -> Json {
    let r = wrap_data(data);

    warp::reply::json(&r)
}

fn response_err(status: u16, error: Error) -> Json {
    let r = wrap_err(status, error);

    warp::reply::json(&r)
}

// https://stackoverflow.com/questions/62964013/how-can-two-headers-of-the-same-name-be-attached-to-a-warp-reply

#[inline]
fn auth_cookie(token: &str) -> String {
    format!(
        // "{}={}; Domain=songday.com; Secure; HttpOnly; Path=/",
        "{}={}; HttpOnly; Path=/",
        val::AUTH_HEADER_NAME,
        token,
    )
}
