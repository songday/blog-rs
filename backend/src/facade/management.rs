use warp::{Rejection, Reply};

use crate::{db::user, service::status};

const CONFIG_HTML: &'static str = include_str!("../resource/page/config.html");
const LOGIN_HTML: &'static str = include_str!("../resource/page/login.html");
const REG_HTML: &'static str = include_str!("../resource/page/reg.html");

pub async fn index(token: Option<String>) -> Result<impl Reply, Rejection> {
    let html;
    if user::have_admin_user().await {
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

pub async fn config(token: Option<String>) -> Result<impl Reply, Rejection> { Ok(warp::reply::html(CONFIG_HTML)) }
