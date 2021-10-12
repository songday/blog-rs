use futures::TryFutureExt;
use warp::{Rejection, Reply};

use crate::facade::asset;
use crate::facade::management;
use crate::service::status;

const INDEX_HTML: &'static str = include_str!("../resource/page/index.html");

pub async fn index(token: Option<String>) -> Result<impl Reply, Rejection> {
    //检查是否有data.db，有则返回前端 index，否则返回设置页面
    if crate::db::management::has_settings().await.unwrap_or(false) {
        Ok(warp::reply::html(INDEX_HTML))
    } else {
        // Ok(warp::redirect::redirect(hyper::Uri::from_static("/management/index")))
        let html = management::SETTINGS_HTML.replace("{{ name }}", "");
        Ok(warp::reply::html(html.as_str()))
    }
}
