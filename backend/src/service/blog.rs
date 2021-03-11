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
    util::io::{self, SupportFileType},
};
use crate::db::model::Tag;
use crate::util::result::Result;

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

