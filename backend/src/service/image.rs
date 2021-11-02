use std::{collections::HashMap, path::Path, sync::Arc};

use bytes::Buf;
use rand::Rng;
use warp::filters::multipart::{FormData, Part};
use blog_common::{
    dto::{
        post::UploadImage,
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
    if file_ext.is_empty() {
        return Err(Error::UploadFailed.into());
    }
    let filename = format!("{}.{}", id, file_ext);
    // let mut file = tokio::fs::File::create(Path::new(&filename)).await?;
    let (mut file, filename) = crate::util::io::get_save_file(id, &filename).await?;
    let b = response.bytes().await?;
    tokio::io::copy_buf(&mut &b[..], &mut file).await?;
    Ok(filename)
}