use std::collections::HashMap;

use blog_common::{
    dto::{git::GitRepositoryInfo, user::UserInfo, Response as ApiResponse},
    result::{Error, ErrorResponse},
    val,
};
use hyper::body::Body;
use hyper::header::{self, HeaderMap, HeaderValue};
use warp::{filters::path::Tail, http::Response, Rejection, Reply};

use crate::{
    db::management,
    db::post,
    facade::{session_id_cookie, wrap_json_data, wrap_json_err},
    service::{export, git, status},
    util::common,
};

static GIT_PAGES_INIT_HTML: &'static str = include_str!("../resource/page/git-pages-init.html");

pub async fn show() -> Result<Response<Body>, Rejection> {
    let setting = management::get_setting(git::SETTING_ITEM_NAME).await?;
    if setting.is_some() {
        let setting = setting.unwrap();
        let r = serde_json::from_str::<GitRepositoryInfo>(&setting.content);
        let html = match r {
            Ok(info) => {
                let mut context = tera::Context::new();
                context.insert("remote_url", &info.remote_url);
                context.insert("name", &info.name);
                context.insert("email", &info.email);
                match export::TEMPLATES.render("git-pages-detail.html", &context) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        format!("Failed render page: {}", e)
                    },
                }
            },
            Err(e) => format!("Failed deserialize information: {}", e),
        };
        Ok(Response::builder().header("Content-Type", "text/html; charset=utf-8").body(html.into()).unwrap())
    } else {
        Ok(Response::builder().header("Content-Type", "text/html; charset=utf-8").body(GIT_PAGES_INIT_HTML.into()).unwrap())
    }
}

pub async fn new_repository(mut params: HashMap<String, String>,) -> Result<impl Reply, Rejection> {
    let empty_str = String::new();
    let url = params.get("url").unwrap_or(&empty_str);
    if !url.starts_with("http") {
        return Ok(super::wrap_json_err(500, Error::BusinessException(String::from("Url must starts with 'http'."))));
    }
    let user = params.get("user").unwrap_or(&empty_str);
    if user.is_empty() {
        return Ok(super::wrap_json_err(500, Error::BusinessException(String::from("UserName must not be empty."))));
    }
    let email = params.get("email").unwrap_or(&empty_str);
    if email.len() < 5 || !common::EMAIL_REGEX.is_match(&email) {
        return Ok(wrap_json_err(500, Error::BusinessException("输入的邮箱地址不合法/Illegal email address.".to_string())));
    }
    let r = url.rfind("/");
    if r.is_none() {
        return Ok(wrap_json_err(500, Error::BusinessException("输入的仓库地址不合法/Illegal repository address.".to_string())));
    }
    let url = params.remove("url").unwrap();
    let user = params.remove("user").unwrap();
    let email = params.remove("email").unwrap();
    let info = GitRepositoryInfo{
        name: user,
        email,
        repository_name: String::from(&url[(r.unwrap() + 1)..]),
        remote_url: url,
        last_export_second: 0,
    };
    match git::new_repository(info).await {
        Ok(_) => Ok(wrap_json_data("")),
        Err(e) => return Ok(wrap_json_err(500, Error::BusinessException(e)))
    }
}

pub async fn push() -> Result<Response<Body>, Rejection> {
    let setting = management::get_setting(git::SETTING_ITEM_NAME).await?;
    let mut message = String::with_capacity(32);
    if setting.is_none() {
        message.push_str("Cannot find git repository setting");
    } else {
        let setting = setting.unwrap();
        let r = serde_json::from_str::<GitRepositoryInfo>(&setting.content);
        if let Ok(info) = r {
            let r = export::git(&info).await;
            let r = git::sync_to_remote(&info);
        } else {
            message.push_str("Cannot deserialize git repository info");
        }
    }
    let r = Response::builder().body(message.into()).unwrap();
    Ok(r)
}
