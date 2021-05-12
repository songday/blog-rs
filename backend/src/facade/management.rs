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
use blog_common::dto::management::SiteData;

const CONFIG_HTML: &'static str = include_str!("../resource/page/config.html");
const LOGIN_HTML: &'static str = include_str!("../resource/page/login.html");
const REG_HTML: &'static str = include_str!("../resource/page/reg.html");

pub async fn index(token: Option<String>) -> Result<impl Reply, Rejection> {
    let html;
    if management::have_admin().await {
        // if matches!(token, Some(t) if status::check_auth(&t).is_ok()) {
        if status::check_auth(token).is_ok() {
            html = CONFIG_HTML;
        } else {
            html = LOGIN_HTML;
        }
    } else {
        html = REG_HTML;
    }
    Ok(warp::reply::html(html))
}

pub async fn admin_register(params: AdminUser) -> Result<impl Reply, Rejection> {
    facade::response(management::admin_register(&params.email, &params.password).await)
}

pub async fn admin_login(token: Option<String>, params: AdminUser) -> Result<impl Reply, Rejection> {
    let token = status::check_verify_code(token, &params.captcha)?;
    facade::response(management::admin_login(&token, &params.email, &params.password).await)
}

pub async fn settings() -> Result<impl Reply, Rejection> { facade::response(management::settings().await) }

pub async fn update_settings(token: Option<String>, setting: Setting) -> Result<impl Reply, Rejection> {
    facade::response(management::update_settings(token, setting).await)
}

pub async fn site_data(token: Option<String>) -> Result<impl Reply, Rejection> {
    let user_info = match status::check_auth(token) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("{:?}", e);
            UserInfo::default()
        },
    };
    let settings = match management::settings().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{:?}", e);
            Setting::default()
        },
    };
    Ok(wrap_json_data(SiteData { settings, user_info }))
}
