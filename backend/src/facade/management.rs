use core::{convert::Infallible, result::Result};

use blog_common::{
    dto::{
        management::{AdminUser, Settings},
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

pub const SETTINGS_HTML: &'static str = include_str!("../resource/page/settings.html");
const LOGIN_HTML: &'static str = include_str!("../resource/page/login.html");

pub async fn index(token: Option<String>) -> Result<impl Reply, Rejection> {
    if status::check_auth(token).is_ok() {
        let settings = management::settings().await?;
        let mut context = tera::Context::new();
        context.insert("name", &settings.name);
        context.insert("domain", &settings.domain);
        context.insert("copyright", &settings.copyright);
        context.insert("license", &settings.license);
        let mut tera = tera::Tera::default();
        let r = match tera.render_str(SETTINGS_HTML, &context) {
            Ok(s) => s,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        Ok(Response::new(r.into()))
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

pub async fn settings() -> Result<impl Reply, Rejection> { facade::response(management::settings().await) }

pub async fn update_settings(token: Option<String>, setting: Settings) -> Result<impl Reply, Rejection> {
    facade::response(management::update_settings(token, setting.into()).await)
}
