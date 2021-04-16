use core::{convert::Infallible, result::Result};

use blog_common::{
    dto::management::{AdminUser, Setting},
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

const CONFIG_HTML: &'static str = include_str!("../resource/page/config.html");
const LOGIN_HTML: &'static str = include_str!("../resource/page/login.html");
const REG_HTML: &'static str = include_str!("../resource/page/reg.html");

pub async fn index(token: Option<String>) -> Result<impl Reply, Rejection> {
    let html;
    if management::have_admin().await {
        if matches!(token, Some(t) if status::check_auth(&t).is_ok()) {
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
