use core::{convert::Infallible, result::Result};

use blog_common::{
    dto::{
        management::{AdminUser, Setting},
        user::UserInfo,
    },
    result::Error,
};
use hyper::{body::Body, header};
use warp::{reply::Response, Rejection, Reply};

use crate::{
    db::management,
    facade,
    facade::{wrap_json_data, wrap_json_err},
    service::status,
    util::common,
};

pub const SETTINGS_HTML: &'static str = include_str!("../resource/page/settings.html");
const LOGIN_HTML: &'static str = include_str!("../resource/page/login.html");

pub fn show_settings_with_fake_auth() -> Response {
    let token = crate::util::common::simple_uuid();
    status::user_online(&token, UserInfo { id: 1 });
    // Ok(warp::redirect::redirect(hyper::Uri::from_static("/management/index")))
    let mut response = warp::reply::Response::new(SETTINGS_HTML.into());
    response.headers_mut().append(
        hyper::header::SET_COOKIE.as_str(),
        super::session_id_cookie(&token).parse().unwrap(),
    );
    response
}

pub async fn index(token: Option<String>) -> Result<impl Reply, Rejection> {
    if status::check_auth(token).is_ok() {
        Ok(Response::new(SETTINGS_HTML.into()))
        // Ok(warp::reply::html(&r))
    } else {
        Ok(Response::new(LOGIN_HTML.into()))
        // Ok(warp::reply::html(LOGIN_HTML))
    }
}

pub async fn admin_login(token: Option<String>, params: AdminUser) -> Result<impl Reply, Rejection> {
    let token = status::check_verify_code(token, &params.captcha)?;
    facade::response(management::admin_login(&token, &params.password).await)
}

pub async fn update_settings(token: Option<String>, setting: Setting) -> Result<impl Reply, Rejection> {
    if let Err(e) = status::check_auth(token) {
        return facade::response(Err(e));
    }
    facade::response(management::update_setting(setting.into()).await)
}

pub async fn forgot_password(authority: Option<warp::host::Authority>) -> Result<impl Reply, Rejection> {
    if let Some(a) = authority {
        if a.host().eq("localhost") || a.host().eq("localhost") {
            return Ok(show_settings_with_fake_auth());
        }
    }
    let mut response = Response::new(
        "请通过localhost或127.0.0.1访问本页/Please visit this page with host: localhost or 127.0.0.1".into(),
    );
    response.headers_mut().append(
        hyper::header::CONTENT_TYPE.as_str(),
        "text/plain; charset=UTF-8".parse().unwrap(),
    );
    Ok(response)
}
