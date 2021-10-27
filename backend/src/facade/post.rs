use core::{convert::Infallible, result::Result};
use std::collections::HashMap;
use std::path::Path;

use blog_common::{
    dto::{post::PostData, user::UserInfo, Response as ApiResponse},
    result::{Error, ErrorResponse},
    val,
};
use bytes::Buf;
use hyper::header::{self, HeaderMap, HeaderValue};
use rand::Rng;
use serde::Serialize;
use sqlx::ColumnIndex;
use warp::{
    filters::multipart::FormData,
    http::{response::Response, StatusCode},
    reply::{Json, Response as WarpResponse},
    Rejection, Reply,
};

use crate::{
    db::post,
    facade::{session_id_cookie, wrap_json_data, wrap_json_err},
    service::status,
    util::common,
};

pub async fn list(mut page_num: u8) -> Result<impl Reply, Rejection> {
    if page_num < 1 {
        page_num = 1;
    }

    match post::list(page_num, 10).await {
        Ok(list) => Ok(wrap_json_data(&list)),
        Err(e) => Ok(wrap_json_err(500, e.0)),
    }
}

pub async fn list_by_tag(tag: String, page_num: u8) -> Result<impl Reply, Rejection> {
    match post::list_by_tag(tag, page_num, 20).await {
        Ok(list) => Ok(wrap_json_data(&list)),
        Err(e) => Ok(wrap_json_err(500, e.0)),
    }
}

pub async fn save(user: Option<UserInfo>, post: PostData) -> Result<impl Reply, Rejection> {
    if user.is_none() {
        return Ok(wrap_json_err(500, Error::NotAuthed));
    }
    match post::save(post).await {
        Ok(blog) => Ok(wrap_json_data(&blog)),
        Err(e) => Ok(wrap_json_err(500, e.0)),
    }
}

pub async fn show(
    token: Option<String>,
    id: u64,
    query_string: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    match post::show(id, query_string).await {
        Ok(mut blog) => {
            blog.editable = status::check_auth(token).is_ok();
            Ok(wrap_json_data(&blog))
        },
        Err(e) => Ok(wrap_json_err(500, e.0)),
    }
}

// https://rust-lang-nursery.github.io/rust-cookbook/web/clients/download.html
pub async fn random_title_image(id: i64) -> crate::util::result::Result<String> {
    let mut rng = rand::thread_rng();
    let url = if rng.gen_range(1..=100) > 70 {
        // https://source.unsplash.com/random/1000x500?keywords.join(",")&sig=cache_buster
        "https://source.unsplash.com/random/1000x500"
    } else {
        "https://picsum.photos/1000/500"
    };
    let response = reqwest::get(url).await?;
    let file_ext = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| {
            // if name.is_empty() { None }
            // else if let Some(p) = name.find(".") {
            //     Some(&name[p..])
            //     // name.find(".")
            //     //     .and_then(|pos| Some(name))
            //     //     .unwrap_or(Some(""))
            // } else { None }
            if name.is_empty() { None }
            else {
                name.find(".")
                    .map(|pos| &name[pos+1..])
            }
        })
        .unwrap_or(
            match response.headers().get("Content-Type") {
                Some(h) => {
                    let r = h.to_str();
                    match r {
                        Ok(header) => {
                            match header {
                                "image/jpeg" => "jpg",
                                "image/png" => "png",
                                _ => ""
                            }
                        },
                        Err(e) => {
                            eprintln!("{:?}", e);
                            ""
                        }
                    }
                },
                None => "",
            }
        );
    let filename = format!("{}.{}", id, file_ext);
    let mut file = tokio::fs::File::create(Path::new(&filename)).await?;
    let b = response.bytes().await?;
    tokio::io::copy_buf(&mut &b[..], &mut file).await?;
    Ok(filename)
}