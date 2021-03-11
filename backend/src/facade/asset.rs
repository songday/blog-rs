use warp::{
    Rejection, Reply,
};

pub async fn index() -> Result<impl Reply, Rejection> {
    let s = include_str!("../asset/page/index.html");
    Ok(warp::reply::html(s))
}

pub async fn about() -> Result<impl Reply, Rejection> {
    let s = include_str!("../asset/page/about.html");
    Ok(warp::reply::html(s))
}
