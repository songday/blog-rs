use futures::TryFutureExt;
use warp::{Rejection, Reply};

use crate::facade::asset;
use crate::facade::management;

async fn index(token: Option<String>) -> Result<impl Reply, Rejection> {
    //检查是否有data.db，有则返回前端 index，否则返回设置页面
    if crate::db::management::has_settings().or_else(false) {
        asset::index()
    } else {
        management::index(token)
    }
}