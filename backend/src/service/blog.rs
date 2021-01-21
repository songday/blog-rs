use std::{collections::HashMap, path::Path, sync::Arc};

use bytes::Buf;
use v_htmlescape;
use warp::filters::multipart::{FormData, Part};

use blog_common::{
    dto::{
        blog::{BlogDetail, NewBlog, UploadImage},
        PaginationData,
    },
    result::Error,
};

use crate::{
    db::blog,
    image::image,
    model::Tag,
    result::Result,
    util::io::{self, SupportFileType},
};

pub async fn list(mut page_num: u8) -> Result<PaginationData<Vec<BlogDetail>>> {
    if page_num < 1 {
        page_num = 1;
    }
    blog::list(page_num, crate::var::BLOG_PAGE_SIZE).await
}

pub async fn tags() -> Result<Vec<Tag>> { blog::tags() }

pub async fn list_by_tag(tag: String, mut page_num: u8) -> Result<PaginationData<Vec<BlogDetail>>> {
    let tag = urlencoding::decode(&tag)?;

    if page_num < 1 {
        page_num = 1;
    }
    blog::list_by_tag(tag, page_num, crate::var::BLOG_PAGE_SIZE).await
}

pub async fn save(mut blog: NewBlog) -> Result<BlogDetail> {
    if blog.title.len() < 3 || blog.title.len() > 60 {
        return Err(Error::SaveBlogFailed.into());
    }
    if blog.content.len() < 5 || blog.content.len() > 65535 {
        return Err(Error::SaveBlogFailed.into());
    }
    blog.content = format!("{}", v_htmlescape::escape(&blog.content));
    blog::save(blog).await
}

pub async fn show(id: u64) -> Result<BlogDetail> {
    if id < 10000 {
        return Err(Error::CannotFoundBlog.into());
    }
    blog::show(id).await
}

pub async fn upload_image(data: FormData) -> Result<UploadImage> {
    let file_info = io::save_upload_file(
        data,
        &[SupportFileType::Png, SupportFileType::Jpg, SupportFileType::Gif],
    )
    .await?;
    let thumbnail = image::resize_from_file(&file_info).await?;
    let d = UploadImage::new(thumbnail, file_info.origin_filename);
    Ok(d)
}

pub async fn save_image(filename: String, body: impl Buf) -> Result<UploadImage> {
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
