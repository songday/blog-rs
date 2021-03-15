use warp::{Rejection, Reply};

pub async fn index() -> Result<impl Reply, Rejection> {
    // let s = include_str!("../asset/page/index.html");
    Ok(warp::reply::html(""))
}
