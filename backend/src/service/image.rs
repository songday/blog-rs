use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use blog_common::{dto::post::UploadImage, result::Error};
use bytes::Buf;
use rand::Rng;
use tokio::io::AsyncWriteExt;
use warp::filters::multipart::{FormData, Part};

use crate::{
    db::{model::Tag, post},
    image::image,
    util::{
        io::{self, SupportFileType},
        result::Result,
    },
};

pub async fn upload(post_id: u64, data: FormData) -> Result<UploadImage> {
    let file_info = io::save_upload_file(
        post_id,
        data,
        &[SupportFileType::Png, SupportFileType::Jpg, SupportFileType::Gif],
    )
    .await?;
    image::resize_from_file(&file_info).await?;
    let d = UploadImage::new(file_info.relative_path, file_info.origin_filename);
    Ok(d)
}

pub async fn get_upload_image(path: &str) -> Result<Vec<u8>> {
    let mut path_buf = PathBuf::with_capacity(32);
    path_buf.push("upload");
    let v: Vec<&str> = path.split_terminator('/').collect();
    for n in v {
        path_buf.push(n);
    }
    match tokio::fs::read(path_buf.as_path()).await {
        Ok(d) => Ok(d),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(Error::UploadFailed.into())
        }
    }
}

pub async fn save(post_id: u64, filename: String, body: impl Buf) -> Result<UploadImage> {
    let filename = urlencoding::decode(&filename)?;

    let file_info = io::save_upload_stream(
        post_id,
        filename,
        body,
        &[SupportFileType::Png, SupportFileType::Jpg, SupportFileType::Gif],
    )
    .await?;
    image::resize_from_file(&file_info).await?;
    let d = UploadImage::new(file_info.relative_path, file_info.origin_filename);
    Ok(d)
}

// pub async fn resize_blog_image<B: AsRef<&[u8]>, T: AsRef<&str>>(b: B, type: T) {}

// https://rust-lang-nursery.github.io/rust-cookbook/web/clients/download.html
pub async fn random_title_image(id: u64) -> Result<String> {
    let url = {
        let mut rng = rand::thread_rng();
        if rng.gen_range(1..=100) > 70 {
            // https://source.unsplash.com/random/1000x500?keywords.join(",")&sig=cache_buster
            "https://source.unsplash.com/random/1000x500"
        } else {
            "https://picsum.photos/1000/500"
        }
    };
    let response = reqwest::get(url).await?;
    let file_ext = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| {
            if name.is_empty() {
                None
            } else {
                name.find(".").map(|pos| &name[pos + 1..])
            }
        })
        .unwrap_or(match response.headers().get("Content-Type") {
            Some(h) => {
                let r = h.to_str();
                match r {
                    Ok(header) => match header {
                        "image/jpeg" => "jpg",
                        "image/png" => "png",
                        _ => "",
                    },
                    Err(e) => {
                        eprintln!("{:?}", e);
                        ""
                    }
                }
            }
            None => "",
        });
    if file_ext.is_empty() {
        return Err(Error::UploadFailed.into());
    }
    let filename = format!("{}.{}", id, file_ext);
    // let mut file = tokio::fs::File::create(Path::new(&filename)).await?;
    let (mut file, _path_buf, relative_path) = crate::util::io::get_save_file(id, &filename).await?;
    let b = response.bytes().await?;
    tokio::io::copy_buf(&mut &b[..], &mut file).await?;
    // file.shutdown()
    Ok(relative_path)
}
