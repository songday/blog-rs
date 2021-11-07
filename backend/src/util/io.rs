use std::{
    cmp::PartialEq,
    hash::Hasher,
    path::{Path, PathBuf},
    str::FromStr,
    vec::Vec,
};
use std::io::Read;

use ahash::AHasher;
use bytes::{Buf, BufMut, BytesMut};
use chrono::prelude::*;
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

// lazy_static! {
//     static ref UPLOAD_DIR_LAYOUT: chrono::format::strftime::StrftimeItems<'static> = chrono::format::strftime::StrftimeItems::new("%Y/%m%d");
// }

#[derive(PartialEq)]
pub enum SupportFileType {
    Gif,
    Jpg,
    Png,
}

impl FromStr for SupportFileType {
    type Err = Error;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s {
            "png" => Ok(SupportFileType::Png),
            "jpg" | "jpeg" => Ok(SupportFileType::Jpg),
            "gif" => Ok(SupportFileType::Gif),
            _ => Err(Error::UnsupportedFileType(String::from(s))),
        }
    }
}

impl From<Option<&str>> for SupportFileType {
    fn from(_: Option<&str>) -> Self { unimplemented!() }
}

pub fn gen_new_upload_filename(post_id: u64, origin_filename: &str, ext: &str) -> String {
    let mut filename = String::with_capacity(128);

    let id = post_id.to_string();
    filename.push_str(&id);
    filename.push('-');

    let now = time::OffsetDateTime::now_utc();
    let mill_sec = now.unix_timestamp().to_string();
    filename.push_str(&mill_sec);
    filename.push('-');

    let mut hasher = AHasher::default();
    hasher.write(origin_filename.as_bytes());
    filename.push_str(hasher.finish().to_string().as_str());
    filename.push('.');
    filename.push_str(ext);

    filename
}

pub async fn get_save_file(
    post_id: u64,
    save_filename: &str,
) -> std::io::Result<(File, PathBuf, String)> {
    let id = post_id.to_string();

    let mut path_buf = PathBuf::with_capacity(128);
    // path_buf.push(val::IMAGE_ROOT_PATH);
    path_buf.push("upload");
    path_buf.push(&id[id.len() - 1..]);
    if !path_buf.as_path().exists() {
        create_dir_all(path_buf.as_path()).await?;
    }

    path_buf.push(save_filename);

    let path = dbg!(path_buf.as_path());

    let f = path.display().to_string();
    // f.insert(0, '/');

    #[cfg(target_os = "windows")]
    let f = f.replace("\\", "/");

    let file = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .open(path)
        .await?;

    Ok((file, path_buf, f))
}

async fn get_upload_file_writer(
    post_id: u64,
    original_filename: &str,
    ext: &str,
) -> std::io::Result<(BufWriter<File>, UploadFileInfo)> {
    let mut upload_file_info = UploadFileInfo::new();
    upload_file_info.origin_filename.push_str(original_filename);
    upload_file_info.extension.push_str(ext);

    let new_filename = gen_new_upload_filename(post_id, &original_filename, ext);

    let (file, save_path_buf, relative_file_path) = get_save_file(post_id, &new_filename).await?;
    upload_file_info.filepath = save_path_buf;
    upload_file_info.relative_path = relative_file_path;

    Ok((
        BufWriter::<File>::with_capacity(32768, file),
        upload_file_info,
    ))
}

fn get_ext<'a, 'b>(filename: &'a str, allow_file_types: &'b [SupportFileType],) -> Result<&'a str> {
    if let Some(ext) = filename.rfind('.').map(|pos| &filename[pos + 1..]) {
        let file_type = ext.parse::<SupportFileType>()?;
        if let None = allow_file_types.into_iter().find(|&t| t == &file_type) {
            return Err(Error::UnsupportedFileType(String::from(ext)));
        }
        Ok(ext)
    } else {
        Err(Error::UnknownFileType)
    }
}

async fn write_buf(writer: &mut BufWriter<File>, mut body: impl Buf) -> Result<usize> {
    let filesize: usize = body.remaining();
    writer.write_all_buf(&mut body).await;
    // let mut b = [0u8; 10240];
    // loop {
    //     let remain = body.remaining();
    //     println!("remain={}", remain);
    //     if remain < 1 {
    //         break;
    //     } else if remain < 10240 {
    //         let mut c = body.chunk();
    //         let _cnt = write_bytes(writer, &c).await?;
    //         break;
    //     } else {
    //         // let mut bytes = body.copy_to_bytes(10240);
    //         body.copy_to_slice(&mut b);
    //         // 下面这个需要放在这里，如果放在下面，那么b的长度始终为0，就会死循环
    //         let cnt = write_bytes(writer, &b).await?;
    //         body.advance(cnt);
    //     }
    // }
    Ok(dbg!(filesize))
}

async fn write_bytes(writer: &mut BufWriter<File>, b: &[u8]) -> Result<usize> {
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
    return Ok(cnt);
}

pub async fn save_upload_file(post_id: u64, data: FormData, allow_file_types: &[SupportFileType]) -> Result<UploadFileInfo> {
    // data need to be: mut data: FormData
    // https://docs.rs/futures/0.3.5/futures/stream/trait.StreamExt.html#method.next
    // loop {
    //     let part = data.next().await;
    //     if let Some(r) = part {
    //     } else {
    //         break;
    //     }
    // }
    // https://docs.rs/futures/0.3.5/futures/stream/trait.StreamExt.html#method.collect
    let parts = data.collect::<Vec<std::result::Result<Part, warp::Error>>>().await;
    let mut filesize = 0usize;
    let mut upload_info: Option<UploadFileInfo> = None;
    let mut writer: Option<BufWriter<File>> = None;
    for r in parts {
        match r {
            Ok(mut p) => {
                if writer.is_none() && p.filename().is_some() {
                    let origin_filename = dbg!(p.filename().unwrap());
                    let ext = get_ext(&origin_filename, allow_file_types)?;

                    match get_upload_file_writer(post_id, origin_filename, ext).await {
                        Ok((w, upload_file_info)) => {
                            writer = Some(w);
                            upload_info = Some(upload_file_info);
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
                        Ok(mut buf) => {
                            match writer {
                                Some(ref mut w) => {
                                    filesize += buf.remaining();
                                    w.write_all_buf(&mut buf).await;
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
        eprintln!("{:?}", e);
        Error::UploadFailed
    })?;

    Ok(upload_info.unwrap())
}

pub async fn save_upload_stream(
    post_id: u64,
    filename: String,
    mut body: impl Buf,
    allow_file_types: &[SupportFileType],
) -> Result<UploadFileInfo> {
    let mut upload_info:Option<UploadFileInfo> = None;

    let ext = get_ext(&filename, allow_file_types)?;

    let mut writer = match get_upload_file_writer(post_id, &filename, ext).await {
        Ok((w, p)) => {
            upload_info = Some(p);
            w
        },
        Err(e) => {
            dbg!(e);
            return Err(Error::UploadFailed);
        },
    };

    let mut upload_info = upload_info.unwrap();
    upload_info.filesize = body.remaining();
    writer.write_all_buf(&mut body).await;

    if let Err(e) = writer.shutdown().await {
        dbg!(&e);
        eprintln!("{:?}", e);
        return Err(Error::UploadFailed);
    }

    Ok(upload_info)
}
