use futures::TryFutureExt;
use warp::{Rejection, Reply};

use crate::facade::asset;
use crate::facade::management;
use crate::service::status;

const INDEX_HTML: &'static str = include_str!("../resource/page/index.html");

pub async fn index(token: Option<String>) -> Result<impl Reply, Rejection> {
    let html;
    //检查是否有data.db，有则返回前端 index，否则返回设置页面
    if crate::db::management::has_settings().await.unwrap_or(false) {
        html = INDEX_HTML;
    } else {
        // Ok(warp::redirect::redirect(hyper::Uri::from_static("/management/index")))
        html = management::SETTINGS_HTML;
    }
    Ok(warp::reply::html(html))
}
