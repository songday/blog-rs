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
use serde::Serialize;
use sqlx::ColumnIndex;
use warp::{
    filters::multipart::FormData,
    http::{response::Response, StatusCode, Uri},
    reply::{Json, Response as WarpResponse},
    Rejection, Reply,
};

use crate::{
    db::post,
    facade::{session_id_cookie, wrap_json_data, wrap_json_err},
    service::{image, status},
    util::common,
};

pub async fn new(token: Option<String>) -> Result<impl Reply, Rejection> {
    if status::check_auth(token).is_err() {
        return Ok(wrap_json_err(500, Error::NotAuthed));
    }
    post::new_post()
        .await
        .map(|id| wrap_json_data(&id))
        .or_else(|e| Ok(wrap_json_err(500, e.0)))
}

pub async fn list(pagination_type: String, post_id: u64) -> Result<impl Reply, Rejection> {
    match post::list(pagination_type.as_str(), post_id, val::POSTS_PAGE_SIZE).await {
        Ok(list) => Ok(wrap_json_data(&list)),
        Err(e) => Ok(wrap_json_err(500, e.0)),
    }
}

pub async fn list_by_tag(tag: String, pagination_type: String, post_id: u64) -> Result<impl Reply, Rejection> {
    match post::list_by_tag(tag, &pagination_type, post_id, val::POSTS_PAGE_SIZE).await {
        Ok(list) => Ok(wrap_json_data(&list)),
        Err(e) => Ok(wrap_json_err(500, e.0)),
    }
}

pub async fn save(user: Option<UserInfo>, post: PostData) -> Result<impl Reply, Rejection> {
    if user.is_none() {
        return Ok(wrap_json_err(403, Error::NotAuthed));
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
    let auth_result = status::check_auth(token);
    let edit = query_string.contains_key("edit");
    if edit && auth_result.is_err() {
        return Ok(wrap_json_err(500, auth_result.unwrap_err().0));
    }
    let editable = auth_result.is_ok() && edit;
    match post::show(id, editable).await {
        Ok(mut blog) => {
            blog.editable = editable;
            Ok(wrap_json_data(&blog))
        },
        Err(e) => Ok(wrap_json_err(500, e.0)),
    }
}

pub async fn delete(id: u64, user: Option<UserInfo>) -> Result<impl Reply, Rejection> {
    if user.is_some() {
        if let Err(e) = image::delete_post_images(id).await {
            eprintln!("{:?}", e);
        } else if let Err(e) = post::delete(id).await {
            eprintln!("{:?}", e);
        }
        // post::delete(id).await.map(|_| wrap_json_data("Deleted")).map_err(|e| wrap_json_err(500, e.0))
    }
    Ok(warp::redirect::found(Uri::from_static("/")))
}
