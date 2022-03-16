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
    service::{export, git::git, status},
    util::common,
};

static GIT_PAGES_INIT_HTML: &'static str = include_str!("../resource/page/git-pages-init.html");

pub async fn show() -> Result<Response<Body>, Rejection> {
    let result = git::must_get_repository_info().await;

    let response = Response::builder().header("Content-Type", "text/html; charset=utf-8");
    let r = if let Ok(info) = result {
        let mut context = tera::Context::new();
        context.insert("remote_url", &info.remote_url);
        context.insert("name", &info.name);
        context.insert("email", &info.email);
        println!("info.branch_name.is_some(): {}", info.branch_name.is_some());
        if info.branch_name.is_some() {
            context.insert("branch", &info.branch_name.unwrap());
            let d: Vec<String> = Vec::new();
            context.insert("branches", &d);
        } else {
            context.insert("branch", "");
            match git::get_branches(&info) {
                Ok(b) => context.insert("branches", &b),
                Err(e) => {
                    return Ok(response
                        .body(format!("Failed getting branches: {:?}", e).into())
                        .unwrap());
                },
            }
        }
        let html = match export::TEMPLATES.render("git-pages-detail.html", &context) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{:?}", e);
                format!("Failed render page: {}", e)
            },
        };
        response.body(html.into()).unwrap()
    } else {
        response.body(GIT_PAGES_INIT_HTML.into()).unwrap()
    };
    Ok(r)
}

pub async fn new_repository(mut params: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    let empty_str = String::new();
    let url = params.get("url").unwrap_or(&empty_str);
    if !url.starts_with("http") {
        return Ok(super::wrap_json_err(
            500,
            Error::BusinessException(String::from("Url must starts with 'http'.")),
        ));
    }
    let user = params.get("user").unwrap_or(&empty_str);
    if user.is_empty() {
        return Ok(super::wrap_json_err(
            500,
            Error::BusinessException(String::from("UserName must not be empty.")),
        ));
    }
    let email = params.get("email").unwrap_or(&empty_str);
    if email.len() < 5 || !common::EMAIL_REGEX.is_match(&email) {
        return Ok(wrap_json_err(
            500,
            Error::BusinessException("输入的邮箱地址不合法/Illegal email address.".to_string()),
        ));
    }
    let mut url = params.remove("url").unwrap();
    if url.ends_with("/") {
        url.pop();
    }
    let r = url.rfind("/");
    if r.is_none() {
        return Ok(wrap_json_err(
            500,
            Error::BusinessException("输入的仓库地址不合法/Illegal repository address.".to_string()),
        ));
    }
    let repository_name = &url[(r.unwrap() + 1)..];
    if repository_name.is_empty() {
        return Ok(wrap_json_err(
            500,
            Error::BusinessException("输入的仓库地址不合法/Illegal repository address.".to_string()),
        ));
    }
    let user = params.remove("user").unwrap();
    let email = params.remove("email").unwrap();
    let info = GitRepositoryInfo {
        name: user,
        email,
        repository_name: String::from(repository_name),
        remote_url: url,
        branch_name: None,
        last_export_second: 0,
    };
    match git::new_repository(info).await {
        Ok(_) => Ok(wrap_json_data("")),
        Err(e) => return Ok(wrap_json_err(500, Error::BusinessException(e))),
    }
}

pub async fn remove_repository() -> Result<impl Reply, Rejection> {
    let result = git::must_get_repository_info().await;
    let message = match result {
        Ok(info) => {
            if let Err(e) = git::remove_repository(info).await {
                format!("Failed remove repository: {}", e)
            } else {
                String::new()
            }
        },
        Err(e) => e,
    };
    if message.is_empty() {
        Ok(super::wrap_json_data(message))
    } else {
        Ok(super::wrap_json_err(500, Error::BusinessException(message)))
    }
}

pub async fn set_branch(tail: Tail) -> Result<impl Reply, Rejection> {
    let result = git::must_get_repository_info().await;
    let message = match result {
        Ok(mut info) => {
            let branch = tail.as_str();
            match git::set_branch(&info, branch) {
                Ok(_) => {
                    info.branch_name = Some(String::from(branch));
                    match git::update_git_repository_info(&info).await {
                        Ok(_) => String::new(),
                        Err(e) => e,
                    }
                },
                Err(e) => format!("Failed switch branch to {} since {}", branch, e),
            }
        },
        Err(e) => e,
    };
    if message.is_empty() {
        Ok(super::wrap_json_data(message))
    } else {
        Ok(super::wrap_json_err(500, Error::BusinessException(message)))
    }
}

pub async fn push() -> Result<impl Reply, Rejection> {
    let result = git::must_get_repository_info().await;
    let message = match result {
        Ok(info) => match crate::service::git::pull::pull(&info) {
            Ok(_) => match export::git(&info).await {
                Ok(_) => match git::sync_to_remote(&info) {
                    Ok(_) => String::new(),
                    Err(e) => format!("Failed to push posts to git: {}", e),
                },
                Err(e) => format!("Failed to export posts: {:?}", e.0),
            },
            Err(e) => format!("Failed pull: {:?}", e),
        },
        Err(e) => e,
    };
    if message.is_empty() {
        Ok(super::wrap_json_data(message))
    } else {
        Ok(super::wrap_json_err(500, Error::BusinessException(message)))
    }
}
