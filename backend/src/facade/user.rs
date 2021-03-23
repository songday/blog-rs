use core::{convert::Infallible, result::Result};

use hyper::header::{self, HeaderMap, HeaderValue};
use warp::{
    reply::{Json, Response as WarpResponse},
    Rejection, Reply,
};

use blog_common::{
    dto::user::{UserInfo, UserInfoWrapper, UserParams},
    result::{Error, ErrorResponse},
    val,
};

use crate::{
    db::user,
    facade::{auth_cookie, response_data, response_err},
    service::status,
    util::common,
};

pub async fn register(params: UserParams) -> Result<impl Reply, Rejection> {
    if params.password1.len() < 3 {
        return Ok(response_err(500, Error::BusinessException("输入的密码不能少于3位".to_string())).into_response());
    }

    if params.email.len() < 5 || !common::EMAIL_REGEX.is_match(&params.email) {
        return Ok(response_err(500, Error::BusinessException("输入的邮箱地址不合法".to_string())).into_response());
    }

    match user::register(&params.email, &params.password1).await {
        Ok(u) => {
            let token = common::simple_uuid();
            status::user_online(&token, u.clone());
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

pub async fn login(token: Option<String>, params: UserParams) -> Result<WarpResponse, Rejection> {
    if token.is_none() {
        return Ok(response_err(500, Error::InvalidSessionId).into_response());
    }

    if params.password1.len() < 3 {
        return Ok(response_err(500, Error::BusinessException("输入的密码不能少于3位".to_string())).into_response());
    }

    if params.email.len() < 5 || !common::EMAIL_REGEX.is_match(&params.email) {
        return Ok(response_err(500, Error::BusinessException("输入的邮箱地址不合法".to_string())).into_response());
    }

    let token = token.unwrap();
    if !status::check_verify_code(&token, &params.captcha) {
        return Ok(response_err(500, Error::InvalidVerifyCode).into_response());
    }

    match user::login(&params.email, &params.password1).await {
        Ok(u) => {
            status::user_online(&token, u.clone());
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

pub async fn logout(token: Option<String>) -> Result<impl Reply, Rejection> {
    if token.is_some() {
        status::user_offline(&token.unwrap());
    }
    Ok(response_data(String::from("Signed out.")))
}

pub async fn info(token: Option<String>) -> Result<impl Reply, Rejection> {
    if token.is_some() {
        return match status::check_auth(&token.unwrap()) {
            Ok(u) => Ok(response_data(u)),
            Err(e) => Ok(response_err(500, e.0)),
        };
    }
    Ok(response_data(String::new()))
}
