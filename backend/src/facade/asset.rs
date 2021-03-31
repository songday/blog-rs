use warp::{Rejection, Reply};

use crate::service::asset;

pub async fn index() -> Result<impl Reply, Rejection> {
    // let s = include_str!("../resource/page/index.html");
    Ok(warp::reply::html(""))
}

pub async fn asset() -> Result<impl Reply, Rejection> {
    // let s = include_str!("../resource/page/index.html");
    Ok(warp::reply::html(""))
}
