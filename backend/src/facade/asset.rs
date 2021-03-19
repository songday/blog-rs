use core::result::Result;

use hyper::{body::Body, header};
use warp::{filters::path::Tail, http::Response, Rejection, Reply};

use crate::service::asset;

pub async fn index() -> Result<impl Reply, Rejection> {
    // let s = include_str!("../resource/page/index.html");
    Ok(warp::reply::html(""))
}

pub async fn get_asset(tail: Tail) -> Result<Response<Body>, Rejection> {
    println!("full path {}", tail.as_str());
    let file = asset::get_asset(tail.as_str());
    if file.is_none() {
        let r = Response::builder().status(404).body("".into()).unwrap();
        Ok(r)
    } else {
        let (name, data) = file.unwrap();
        let r = Response::builder()
            .header(header::CONTENT_TYPE, &asset::get_content_type(name))
            .header(header::CONTENT_LENGTH, data.len())
            .header(header::CONTENT_ENCODING, "gzip")
            .body(data.into())
            .unwrap();
        Ok(r)
    }
}
