use std::{
    cmp::PartialEq,
    hash::Hasher,
    path::{Path, PathBuf},
    str::FromStr,
    vec::Vec,
};

use ahash::AHasher;
use bytes::{Buf, BufMut, BytesMut};
use chrono::{format::strftime::StrftimeItems, prelude::*};
use futures::StreamExt;
use image::ImageFormat;
use lazy_static::lazy_static;
use tokio::{
    fs::{create_dir_all, rename, write, File, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
};
use warp::filters::multipart::{FormData, Part};

use blog_common::{
    dto::UploadFileInfo,
    result::{Error, Result},
};

use crate::var;

lazy_static! {
    static ref UPLOAD_DIR_LAYOUT: StrftimeItems<'static> = StrftimeItems::new("%Y/%m/%d");
}

#[derive(PartialEq)]
pub enum SupportFileType {
    Gif,
    Jpg,
    Jpeg,
    Png,
    Html,
}

impl FromStr for SupportFileType {
    type Err = Error;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s {
            "png" => Ok(SupportFileType::Png),
            "jpg" | "jpeg" => Ok(SupportFileType::Jpg),
            "gif" => Ok(SupportFileType::Gif),
            "htm" | "html" => Ok(SupportFileType::Html),
            _ => Err(Error::UnsupportedFileType(String::from(s))),
        }
    }
}

impl From<Option<&str>> for SupportFileType {
    fn from(_: Option<&str>) -> Self { unimplemented!() }
}

async fn get_upload_file_writer(
    original_filename: &str,
    ext: &str,
) -> std::io::Result<(BufWriter<File>, PathBuf, usize)> {
    let now = chrono::offset::Utc::now();
    let year = now.year().to_string();
    let month = now.month().to_string();
    let day = now.day().to_string();

    let mut path_buf = PathBuf::with_capacity(128);
    path_buf.push(var::IMAGE_ROOT_PATH);
    path_buf.push(year.as_str());
    path_buf.push(month.as_str());
    path_buf.push(day.as_str());
    if !path_buf.as_path().exists() {
        create_dir_all(path_buf.as_path()).await?;
    }

    let mut filename = String::with_capacity(128);
    filename.push_str(year.as_str());
    filename.push_str(month.as_str());
    filename.push_str(day.as_str());
    filename.push_str(now.timestamp_subsec_nanos().to_string().as_str());
    let mut hasher = AHasher::default();
    hasher.write(original_filename.as_bytes());
    filename.push_str(hasher.finish().to_string().as_str());

    let new_filename_len = path_buf.as_path().to_str().unwrap().len() + filename.len();

    filename.push_str("_original");
    path_buf.push(filename.as_str());
    path_buf.set_extension(ext);
    dbg!(&path_buf);

    let file = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .open(path_buf.as_path())
        .await?;

    Ok((
        BufWriter::<File>::with_capacity(32768, file),
        path_buf,
        new_filename_len,
    ))
}

fn ext(filename: &str) -> Option<&str> {
    let p = filename.rfind('.');
    if p.is_none() {
        return None;
    }
    Some(&filename[p.unwrap() + 1..])
}

async fn write_buf(writer: &mut BufWriter<File>, mut body: impl Buf) -> Result<usize> {
    let mut filesize: usize = 0;
    let mut b = [0u8; 10240];
    while body.has_remaining() {
        // let mut bytes = body.copy_to_bytes(10240);
        body.copy_to_slice(&mut b);
        // 下面这个需要放在这里，如果放在下面，那么b的长度始终为0，就会死循环
        let cnt = b.len();
        let mut pos = 0;
        while pos < cnt {
            match writer.write(&b[pos..]).await {
                Ok(c) => pos += c,
                Err(e) => {
                    dbg!(e);
                    return Err(Error::UploadFailed);
                },
            };
        }
        filesize = filesize + cnt;
        body.advance(cnt);
    }
    dbg!(filesize);
    Ok(filesize)
}

pub async fn save_upload_file(data: FormData, allow_file_types: &[SupportFileType]) -> Result<UploadFileInfo> {
    // data need to be: mut data: FormData
    // https://docs.rs/futures/0.3.5/futures/stream/trait.StreamExt.html#method.next
    // loop {
    //     let part = data.next().await;
    //     if let Some(r) = part {
    //         //todo
    //     } else {
    //         break;
    //     }
    // }
    // https://docs.rs/futures/0.3.5/futures/stream/trait.StreamExt.html#method.collect
    let parts = data.collect::<Vec<std::result::Result<Part, warp::Error>>>().await;
    let mut filesize = 0usize;
    let mut upload_info = UploadFileInfo::new();
    let mut writer: Option<BufWriter<File>> = None;
    for r in parts {
        match r {
            Ok(mut p) => {
                if writer.is_none() && p.filename().is_some() {
                    let origin_filename = dbg!(p.filename().unwrap());
                    let ext = ext(&origin_filename);
                    if ext.is_none() {
                        return Err(Error::UnknownFileType);
                    }
                    let ext = ext.unwrap();
                    let file_type = ext.parse::<SupportFileType>()?;
                    if let None = allow_file_types.into_iter().find(|&t| t == &file_type) {
                        return Err(Error::UnsupportedFileType(String::from(ext)));
                    }
                    upload_info.origin_filename.push_str(origin_filename);
                    upload_info.extension.push_str(ext);

                    match get_upload_file_writer(origin_filename, ext).await {
                        Ok((w, p, new_filename_len)) => {
                            writer = Some(w);
                            upload_info.filepath.push(p);
                            upload_info.new_filename_len = new_filename_len;
                        },
                        Err(e) => {
                            dbg!(e);
                            return Err(Error::UploadFailed);
                        },
                    };
                }
                if (&writer).is_none() {
                    continue;
                }
                if let Some(r) = p.data().await {
                    match r {
                        Ok(buf) => {
                            match writer {
                                Some(ref mut w) => {
                                    filesize += write_buf(w, buf).await?;
                                },
                                None => {},
                            };
                        },
                        Err(e) => {
                            dbg!(e);
                            return Err(Error::UploadFailed);
                        },
                    };
                }
            },
            Err(e) => {
                dbg!(e);
                return Err(Error::UploadFailed);
            },
        };
    }

    if filesize == 0 {
        return Err(Error::UploadFailed);
    }

    writer.unwrap().shutdown().await.map_err(|e| {
        dbg!(&e);
        eprintln!("{}", e);
        Error::UploadFailed
    })?;

    Ok(upload_info)
}

pub async fn save_upload_stream(
    filename: String,
    body: impl Buf,
    allow_file_types: &[SupportFileType],
) -> Result<UploadFileInfo> {
    let mut upload_info = UploadFileInfo::new();

    let ext = ext(&filename);
    if ext.is_none() {
        return Err(Error::UnknownFileType);
    }

    let ext = ext.unwrap();
    let file_type = ext.parse::<SupportFileType>()?;
    if let None = allow_file_types.into_iter().find(|&t| t == &file_type) {
        return Err(Error::UnsupportedFileType(String::from(ext)));
    }

    let mut writer = match get_upload_file_writer(&filename, ext).await {
        Ok((w, p, new_filename_len)) => {
            upload_info.filepath.push(p);
            upload_info.new_filename_len = new_filename_len;
            w
        },
        Err(e) => {
            dbg!(e);
            return Err(Error::UploadFailed);
        },
    };

    upload_info.filesize = write_buf(&mut writer, body).await?;

    match writer.shutdown().await {
        Ok(t) => {},
        Err(e) => {
            dbg!(&e);
            eprintln!("{}", e);
            return Err(Error::UploadFailed);
        },
    };

    upload_info.extension.push_str(ext);
    upload_info.origin_filename.push_str(&filename);

    Ok(upload_info)
}
