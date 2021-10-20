use blog_common::dto::user::UserInfo;
use futures::TryFutureExt;
use warp::{Rejection, Reply};

use crate::facade::asset;
use crate::facade::management;
use crate::service::status;

pub(crate) const INDEX_HTML: &'static str = include_str!("../resource/page/index.html");

pub async fn index() -> Result<impl Reply, Rejection> {
    //检查是否有data.db，有则返回前端 index，否则返回设置页面
    if crate::db::management::has_settings().await.unwrap_or(false) {
        Ok(warp::reply::html(INDEX_HTML).into_response())
        // Ok(warp::reply::Response::new(INDEX_HTML.into()))
    } else {
        let token = crate::util::common::simple_uuid();
        status::user_online(&token, UserInfo { id:1 });
        // Ok(warp::redirect::redirect(hyper::Uri::from_static("/management/index")))
        let html = management::SETTINGS_HTML.replace("{{ name }}", "")
            .replace("{{ domain }}", "").replace("{{ copyright }}", "")
            .replace("{{ license }}", "");
        let mut response = warp::reply::Response::new(html.into());
        response.headers_mut().append(hyper::header::SET_COOKIE.as_str(), super::session_id_cookie(&token).parse().unwrap());
        Ok(response)
    }
}
