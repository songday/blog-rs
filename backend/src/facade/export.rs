use blog_common::{
    dto::{git::GitRepositoryInfo, user::UserInfo, Response as ApiResponse},
    result::{Error, ErrorResponse},
    val,
};
use hyper::body::Body;
use hyper::header::{self, HeaderMap, HeaderValue};
use warp::{filters::path::Tail, http::Response, Rejection, Reply};

use crate::{
    db::management,
    db::post,
    facade::{session_id_cookie, wrap_json_data, wrap_json_err},
    service::{export, status},
    util::common,
};

pub async fn export_handler(tail: Tail, user: Option<UserInfo>) -> Result<Response<Body>, Rejection> {
    if user.is_none() {
        return Ok(Response::builder().status(403).body("".into()).unwrap());
    }
    let path = tail.as_str();
    if path.eq("hugo") {
        return hugo().await;
    }
    if path.rfind(".zip").is_some() {
        return Ok(get_file(path));
    }
    Ok(Response::builder().status(404).body("".into()).unwrap())
}

fn get_file(file: &str) -> Response<Body> {
    let file = std::env::current_dir().unwrap().join("export").join(file);
    if file.exists() {
        match std::fs::read(file.as_path()) {
            Ok(d) => {
                return Response::builder()
                    .header(header::CONTENT_TYPE, "application/octet-stream")
                    .header(header::CONTENT_LENGTH, d.len())
                    .body(d.into())
                    .unwrap()
            },
            Err(e) => {
                eprintln!("{:?}", e);
            },
        }
    }
    Response::builder().status(404).body("".into()).unwrap()
}

async fn hugo() -> Result<Response<Body>, Rejection> {
    let filename = export::hugo().await?;
    let mut uri = String::with_capacity(64);
    uri.push_str("/export/");
    uri.push_str(&filename);
    // Ok(warp::redirect::redirect(warp::http::Uri::from_static(&uri)))
    let r = Response::builder()
        .header(header::CONTENT_TYPE, "text/plain")
        .header(header::CONTENT_LENGTH, uri.len())
        .body(uri.into())
        .unwrap();
    Ok(r)
}
