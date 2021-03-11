use core::{convert::Infallible, result::Result};

use bytes::Buf;
use hyper::header::{self, HeaderMap, HeaderValue};
use serde::Serialize;
use warp::{
    filters::multipart::FormData,
    http::{response::Response, StatusCode},
    reply::{Json, Response as WarpResponse},
    Rejection, Reply,
};

use blog_common::{
    dto::{
        post::NewPost,
        user::{LoginParams, RegisterParams, UserInfo, UserInfoWrapper},
        Response as ApiResponse,
    },
    result::{Error, ErrorResponse},
    var,
};

use crate::{
    db::post,
    facade::{auth_cookie, response_data, response_err},
    util::common,
    service::status,
};

pub async fn list(mut page_num: u8) -> Result<impl Reply, Rejection> {
    if page_num < 1 {
        page_num = 1;
    }

    match post::list(page_num, 20).await {
        Ok(list) => Ok(response_data(&list)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}

pub async fn list_by_tag(tag: String, page_num: u8) -> Result<impl Reply, Rejection> {
    match post::list_by_tag(tag, page_num, 20).await {
        Ok(list) => Ok(response_data(&list)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}

pub async fn save(user: Option<UserInfo>, post: NewPost) -> Result<impl Reply, Rejection> {
    if user.is_none() {
        return Ok(response_err(500, Error::NotAuthed));
    }
    match post::save(post).await {
        Ok(blog) => Ok(response_data(&blog)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}

pub async fn show(id: u64) -> Result<impl Reply, Rejection> {
    match post::show(id).await {
        Ok(blog) => Ok(response_data(&blog)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}
