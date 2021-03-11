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
        user::{LoginParams, RegisterParams, UserInfo, UserInfoWrapper},
        Response as ApiResponse,
    },
    result::{Error, ErrorResponse},
    var,
};

use crate::{
    db::tag,
    facade::{auth_cookie, response_data, response_err},
    util::common,
    service::status,
};

pub async fn list() -> Result<impl Reply, Rejection> {
    match tag::list().await {
        Ok(list) => Ok(response_data(&list)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}
