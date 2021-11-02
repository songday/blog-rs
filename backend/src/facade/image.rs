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
    dto::{post::UploadImage, user::UserInfo},
    result::{Error, ErrorResponse},
    val,
};

use crate::{
    db::user,
    facade::{session_id_cookie, wrap_json_data, wrap_json_err},
    image::image,
    service::status,
    util::{
        common,
        io::{self, SupportFileType},
    },
};

pub async fn verify_image(token: Option<String>) -> Result<WarpResponse, Rejection> {
    let token = token.unwrap_or(common::simple_uuid());
    dbg!(&token);
    match status::get_verify_code(&token) {
        Ok(n) => {
            let b = crate::image::image::gen_verify_image(n.as_slice());
            let mut r = Response::new(b.into());
            let mut header = HeaderMap::with_capacity(2);
            header.insert(header::CONTENT_TYPE, HeaderValue::from_str("image/png").unwrap());
            header.insert(
                header::SET_COOKIE,
                HeaderValue::from_str(&session_id_cookie(&token)).unwrap(),
            );
            // header.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_str("*").unwrap());
            // header.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_str("true").unwrap());
            let headers = r.headers_mut();
            headers.extend(header);
            Ok(r)
        },
        Err(e) => return Ok(Response::new("Wrong request token".into())),
    }
}

pub async fn upload(user: Option<UserInfo>, data: FormData) -> Result<impl Reply, Rejection> {
    if user.is_none() {
        return Ok(wrap_json_err(500, Error::NotAuthed));
    }
    let file_info = io::save_upload_file(
        data,
        &[SupportFileType::Png, SupportFileType::Jpg, SupportFileType::Gif],
    )
    .await;
    if let Err(e) = file_info {
        return Ok(wrap_json_err(500, e));
    }
    let file_info = file_info.unwrap();
    let thumbnail = image::resize_from_file(&file_info).await?;
    let d = UploadImage::new(thumbnail, file_info.origin_filename);
    Ok(wrap_json_data(&d))
}

pub async fn save(filename: String, user: Option<UserInfo>, body: impl Buf) -> Result<impl Reply, Rejection> {
    if user.is_none() {
        return Ok(wrap_json_err(500, Error::NotAuthed));
    }
    let filename = match urlencoding::decode(&filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{:#?}", e);
            return Ok(wrap_json_err(500, Error::BadRequest));
        },
    };
    let file_info = io::save_upload_stream(
        filename,
        body,
        &[SupportFileType::Png, SupportFileType::Jpg, SupportFileType::Gif],
    )
    .await;
    if let Err(e) = file_info {
        return Ok(wrap_json_err(500, e));
    }
    let file_info = file_info.unwrap();
    let thumbnail = image::resize_from_file(&file_info).await?;
    let d = UploadImage::new(thumbnail, file_info.origin_filename);
    Ok(wrap_json_data(&d))
}

// pub async fn resize_blog_image<B: AsRef<&[u8]>, T: AsRef<&str>>(b: B, type: T) {}

pub async fn random_title_image(id: i64) -> Result<impl Reply, Rejection> {
    crate::service::image::random_title_image(id).await
        .map(|f| wrap_json_data(&f))
        .or_else(|e| Ok(wrap_json_err(500, e.0)))
}