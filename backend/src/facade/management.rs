use core::{convert::Infallible, result::Result};
use std::collections::HashMap;

use blog_common::{
    dto::{
        management::{AdminUser, Setting},
        user::UserInfo,
    },
    result::Error,
};
use hyper::{body::Body, header};
use warp::{http::Uri, reply::Response, Rejection, Reply};

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
    let token = common::simple_uuid();
    status::user_online(&token, UserInfo { id: 1 });
    // Ok(warp::redirect::redirect(hyper::Uri::from_static("/management/index")))
    let mut response = warp::reply::Response::new(SETTINGS_HTML.into());
    response.headers_mut().append(
        header::SET_COOKIE.as_str(),
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
        header::CONTENT_TYPE.as_str(),
        "text/plain; charset=UTF-8".parse().unwrap(),
    );
    Ok(response)
}

pub async fn show_render_templates_page(token: Option<String>) -> Result<warp::http::Response<Body>, Rejection> {
    if status::check_auth(token).is_err() {
        return Ok(super::management_sign_in("/management/templates").into_response());
    }
    let response = warp::http::Response::builder().header("Content-Type", "text/html; charset=utf-8");
    let setting = match management::get_setting("post_detail_render_template").await {
        Ok(s) => s,
        Err(e) => return Ok(response.body(format!("{:?}", e.0).into()).unwrap()),
    };
    let mut context = tera::Context::new();
    if let Some(setting) = setting {
        context.insert("post_detail_template", &setting.content);
    }
    let html = match crate::service::export::TEMPLATES.render("render-template.html", &context) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{:?}", e);
            format!("Failed render page: {}", e)
        },
    };
    Ok(response.body(html.into()).unwrap())
}

pub async fn update_render_templates(
    token: Option<String>,
    data: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    if let Err(e) = status::check_auth(token) {
        return facade::response(Err(e));
    }
    let item = "post_detail_render_template";
    let setting = crate::db::model::Setting {
        item: item.to_string(),
        content: data.get(item).map_or(String::new(), |s| String::from(s)),
    };
    match management::update_setting(setting).await {
        Ok(_) => facade::response(Ok("")),
        Err(e) => facade::response(Err(e)),
    }
}
