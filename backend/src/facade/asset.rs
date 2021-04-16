use core::result::Result;

use hyper::{body::Body, header};
use warp::{filters::path::Tail, http::Response, Rejection, Reply};

use crate::service::asset;

pub async fn index() -> Result<impl Reply, Rejection> { Ok(response_asset("index.html")) }

pub async fn get_asset(tail: Tail) -> Result<Response<Body>, Rejection> { Ok(response_asset(tail.as_str())) }

fn response_asset(asset: &str) -> Response<Body> {
    let file = asset::get_asset(asset);
    if file.is_none() {
        Response::builder().status(404).body("".into()).unwrap()
    } else {
        let (_name, data, mime) = file.unwrap();
        let r = Response::builder()
            .header(header::CONTENT_TYPE, mime)
            .header(header::CONTENT_LENGTH, data.len())
            .header(header::CONTENT_ENCODING, "gzip")
            .body(data.into())
            .unwrap();
        r
    }
}
