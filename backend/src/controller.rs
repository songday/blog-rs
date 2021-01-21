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
        blog::NewBlog,
        user::{LoginParams, RegisterParams, UserInfo, UserInfoWrapper},
        Response as ApiResponse,
    },
    result::{Error, ErrorResponse},
    var,
};

use crate::{
    service::{blog, status, user},
    util::common,
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

pub async fn index() -> Result<impl Reply, Rejection> {
    let s = include_str!("asset/page/index.html");
    Ok(warp::reply::html(s))
}

pub async fn about() -> Result<impl Reply, Rejection> {
    let s = include_str!("asset/page/about.html");
    Ok(warp::reply::html(s))
}

// https://stackoverflow.com/questions/62964013/how-can-two-headers-of-the-same-name-be-attached-to-a-warp-reply

#[inline]
fn auth_cookie(token: &str) -> String {
    format!(
        // "{}={}; Domain=songday.com; Secure; HttpOnly; Path=/",
        "{}={}; HttpOnly; Path=/",
        var::AUTH_HEADER_NAME,
        token,
    )
}

pub async fn verify_image(token: Option<String>) -> Result<WarpResponse, Rejection> {
    let token = token.unwrap_or(common::simple_uuid());
    dbg!(&token);
    match status::get_verify_code(&token) {
        Ok(n) => {
            let b = crate::image::image::gen_verify_image(n.as_slice());
            let mut r = Response::new(b.into());
            let mut header = HeaderMap::with_capacity(1);
            header.insert(header::CONTENT_TYPE, HeaderValue::from_str("image/png").unwrap());
            header.insert(header::SET_COOKIE, HeaderValue::from_str(&auth_cookie(&token)).unwrap());
            // header.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_str("*").unwrap());
            // header.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_str("true").unwrap());
            let headers = r.headers_mut();
            headers.extend(header);
            Ok(r)
        },
        Err(e) => return Ok(Response::new("Wrong request token".into())),
    }
}

pub async fn user_register(token: Option<String>, params: RegisterParams) -> Result<impl Reply, Rejection> {
    if token.is_none() {
        return Ok(response_err(500, Error::InvalidSessionId).into_response());
    }

    let token = token.unwrap();
    if !status::check_verify_code(&token, &params.captcha) {
        return Ok(response_err(500, Error::InvalidVerifyCode).into_response());
    }

    if !"webmaster@songday.com".eq(&params.email) {
        return Ok(response_err(500, Error::RegisterFailed).into_response());
    }

    match user::register(&token, &params.email, &params.password1).await {
        Ok(u) => {
            let w = UserInfoWrapper {
                user_info: u,
                access_token: token,
            };
            let reply = response_data(&w);
            let reply_with_header =
                warp::reply::with_header(reply, header::SET_COOKIE.as_str(), auth_cookie(&w.access_token));
            Ok(reply_with_header.into_response())
        },
        Err(e) => {
            let reply = response_err(500, e.0);
            Ok(reply.into_response())
        },
    }
}

pub async fn user_login(token: Option<String>, params: LoginParams) -> Result<WarpResponse, Rejection> {
    if token.is_none() {
        return Ok(response_err(500, Error::InvalidSessionId).into_response());
    }

    let token = token.unwrap();
    if !status::check_verify_code(&token, &params.captcha) {
        return Ok(response_err(500, Error::InvalidVerifyCode).into_response());
    }

    match user::login(&token, &params.email, &params.password).await {
        Ok(u) => {
            let w = UserInfoWrapper {
                user_info: u,
                access_token: token,
            };
            let reply = response_data(&w);
            let reply_with_header =
                warp::reply::with_header(reply, header::SET_COOKIE.as_str(), auth_cookie(&w.access_token));
            Ok(reply_with_header.into_response())
        },
        Err(e) => {
            let reply = response_err(500, e.0);
            Ok(reply.into_response())
        },
    }
}

pub async fn user_logout(token: Option<String>) -> Result<impl Reply, Rejection> {
    if token.is_some() {
        status::user_offline(&token.unwrap());
    }
    Ok(response_data(String::from("Signed out.")))
}

pub async fn user_info(token: Option<String>) -> Result<impl Reply, Rejection> {
    if token.is_some() {
        return match status::check_auth(&token.unwrap()) {
            Ok(u) => Ok(response_data(u)),
            Err(e) => Ok(response_err(500, e.0)),
        };
    }
    Ok(response_data(String::new()))
}

pub async fn blog_list(page_num: u8) -> Result<impl Reply, Rejection> {
    match blog::list(page_num).await {
        Ok(list) => Ok(response_data(&list)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}

pub async fn blog_tags() -> Result<impl Reply, Rejection> {
    match blog::tags().await {
        Ok(list) => Ok(response_data(&list)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}

pub async fn blog_list_by_tag(tag: String, page_num: u8) -> Result<impl Reply, Rejection> {
    match blog::list_by_tag(tag, page_num).await {
        Ok(list) => Ok(response_data(&list)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}

pub async fn blog_save(user: Option<UserInfo>, blog: NewBlog) -> Result<impl Reply, Rejection> {
    if user.is_none() {
        return Ok(response_err(500, Error::NotAuthed));
    }
    match blog::save(blog).await {
        Ok(blog) => Ok(response_data(&blog)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}

pub async fn blog_show(id: u64) -> Result<impl Reply, Rejection> {
    match blog::show(id).await {
        Ok(blog) => Ok(response_data(&blog)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}

pub async fn blog_upload_image(user: Option<UserInfo>, data: FormData) -> Result<impl Reply, Rejection> {
    if user.is_none() {
        return Ok(response_err(500, Error::NotAuthed));
    }
    match blog::upload_image(data).await {
        Ok(d) => Ok(response_data(&d)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}

pub async fn blog_save_image(
    filename: String,
    user: Option<UserInfo>,
    body: impl Buf,
) -> Result<impl Reply, Rejection> {
    if user.is_none() {
        return Ok(response_err(500, Error::NotAuthed));
    }
    match blog::save_image(filename, body).await {
        Ok(d) => Ok(response_data(&d)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}
