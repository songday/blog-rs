use std::{collections::HashMap, path::Path, sync::Arc};

use bytes::Buf;
use v_htmlescape;
use warp::filters::multipart::{FormData, Part};

use blog_common::{
    dto::{
        post::{PostData, PostDetail, UploadImage},
        PaginationData,
    },
    result::Error,
};

use crate::{
    db::{model::Tag, post},
    image::image,
    util::{
        io::{self, SupportFileType},
        result::Result,
    },
};

pub async fn upload(data: FormData) -> Result<UploadImage> {
    let file_info = io::save_upload_file(
        data,
        &[SupportFileType::Png, SupportFileType::Jpg, SupportFileType::Gif],
    )
    .await?;
    let thumbnail = image::resize_from_file(&file_info).await?;
    let d = UploadImage::new(thumbnail, file_info.origin_filename);
    Ok(d)
}

pub async fn save(filename: String, body: impl Buf) -> Result<UploadImage> {
    let filename = urlencoding::decode(&filename)?;

    let file_info = io::save_upload_stream(
        filename,
        body,
        &[SupportFileType::Png, SupportFileType::Jpg, SupportFileType::Gif],
    )
    .await?;
    let thumbnail = image::resize_from_file(&file_info).await?;
    let d = UploadImage::new(thumbnail, file_info.origin_filename);
    Ok(d)
}

// pub async fn resize_blog_image<B: AsRef<&[u8]>, T: AsRef<&str>>(b: B, type: T) {}
