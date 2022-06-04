use blog_common::dto::user::UserInfo;
use futures::TryFutureExt;
use warp::{Rejection, Reply};

use crate::facade::asset;
use crate::facade::management;
use crate::service::status;

pub(crate) const INDEX_HTML: &'static str = include_str!("../resource/page/index.html");

pub async fn index() -> Result<impl Reply, Rejection> {
    //检查是否有data.db，有则返回前端 index，否则返回设置页面
    if crate::db::management::has_admin_password().await.unwrap_or(false) {
        Ok(warp::reply::html(INDEX_HTML).into_response())
        // Ok(warp::reply::Response::new(INDEX_HTML.into()))
    } else {
        // Ok(warp::redirect::temporary(hyper::Uri::from_static("/management/index")))
        let response = management::show_settings_with_fake_auth();
        Ok(response)
    }
}
