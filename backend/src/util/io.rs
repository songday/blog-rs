use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::FileType;
use std::io::Read;
use std::{
    cmp::PartialEq,
    hash::Hasher,
    path::{Path, PathBuf},
    str::FromStr,
    vec::Vec,
};

use ahash::AHasher;
use blog_common::dto::TextFieldInfo;
use blog_common::{
    dto::{FormDataItem, UploadFileInfo},
    result::{Error, Result},
    util::time,
};
use bytes::{Buf};
use futures::StreamExt;
use tokio::{
    fs::{create_dir_all, rename, write, File, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
};
use warp::filters::multipart::{FormData, Part};

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
    fn from(_: Option<&str>) -> Self {
        unimplemented!()
    }
}

pub async fn get_save_path(
    post_id: u64,
    origin_filename: &str,
    ext: &str,
    rename: bool,
) -> std::io::Result<(PathBuf, String)> {
    let id = post_id.to_string();

    // 生成子目录
    let mut path_buf = PathBuf::with_capacity(128);
    // path_buf.push(val::IMAGE_ROOT_PATH);
    path_buf.push("upload");
    path_buf.push(&id[id.len() - 1..]);
    if !path_buf.as_path().exists() {
        create_dir_all(path_buf.as_path()).await?;
    }

    if rename {
        // 新文件名
        let mut filename = String::with_capacity(128);
        filename.push_str(&id);
        filename.push('-');

        let mill_sec = time::unix_epoch_sec().to_string();
        filename.push_str(&mill_sec);
        filename.push('-');

        let mut hasher = AHasher::default();
        hasher.write(origin_filename.as_bytes());
        filename.push_str(hasher.finish().to_string().as_str());
        filename.push('.');
        filename.push_str(ext);

        path_buf.push(&filename);
    } else {
        path_buf.push(origin_filename);
    }

    let path = dbg!(path_buf.as_path());

    let f = path.display().to_string();
    // f.insert(0, '/');

    #[cfg(target_os = "windows")]
    let f = f.replace("\\", "/");

    Ok((path_buf, f))
}

pub async fn get_save_file(
    post_id: u64,
    origin_filename: &str,
    ext: &str,
    rename: bool,
) -> std::io::Result<(File, PathBuf, String)> {
    let (path_buf, path_str) = get_save_path(post_id, origin_filename, ext, rename).await?;

    let file = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .open(path_buf.as_path())
        .await?;

    Ok((file, path_buf, path_str))
}

async fn get_upload_file_writer(
    post_id: u64,
    original_filename: &str,
    ext: &str,
) -> std::io::Result<(BufWriter<File>, UploadFileInfo)> {
    let mut upload_file_info = UploadFileInfo::new();
    upload_file_info.original_filename.push_str(original_filename);
    upload_file_info.extension.push_str(ext);

    let (file, save_path_buf, relative_file_path) = get_save_file(post_id, &original_filename, ext, true).await?;
    upload_file_info.filepath = save_path_buf;
    upload_file_info.relative_path = relative_file_path;

    Ok((BufWriter::<File>::with_capacity(32768, file), upload_file_info))
}

fn get_ext<'a, 'b>(filename: &'a str, allow_file_types: &'b [SupportFileType]) -> Result<&'a str> {
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

pub async fn save_upload_file(
    post_id: u64,
    data: FormData,
    allow_file_types: &[SupportFileType],
) -> Result<Vec<FormDataItem>> {
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
    let mut form_data_result: Vec<FormDataItem> = Vec::with_capacity(parts.len());
    let mut file_writers: HashMap<String, (BufWriter<File>, UploadFileInfo)> = HashMap::with_capacity(8);
    let mut filesize = 0usize;
    // let mut upload_info: Option<UploadFileInfo> = None;
    // let mut writer: Option<BufWriter<File>> = None;
    for r in parts {
        match r {
            Ok(mut p) => {
                if p.name().eq("is-title-image") {
                    let d = p.data().await.unwrap().unwrap();
                    let b = d.chunk();
                    // let d = String::from_utf8(d).unwrap();
                    let n = i32::from_be_bytes(<[u8; 4]>::try_from(b).unwrap());
                }
                if p.filename().is_some() {
                    let filename = p.filename().unwrap();
                    if !file_writers.contains_key(filename) {
                        let ext = get_ext(filename, allow_file_types)?;
                        match get_upload_file_writer(post_id, filename, ext).await {
                            Ok(new_data) => {
                                file_writers.insert(filename.to_string(), new_data);
                            },
                            Err(e) => {
                                dbg!(e);
                                return Err(Error::UploadFailed);
                            },
                        }
                    }
                    let (w, i) = file_writers.get_mut(filename).unwrap();
                    if let Some(r) = p.data().await {
                        match r {
                            Ok(mut buf) => {
                                filesize += buf.remaining();
                                w.write_all_buf(&mut buf).await;
                            },
                            Err(e) => {
                                dbg!(e);
                                return Err(Error::UploadFailed);
                            },
                        };
                    }
                } else {
                    let d = p.data().await.unwrap().unwrap();
                    let b = d.chunk();
                    let s = String::from_utf8(b.to_vec()).unwrap();
                    let form_data_item = FormDataItem::TEXT(TextFieldInfo {
                        name: p.name().to_string(),
                        value: s,
                    });
                    form_data_result.push(form_data_item);
                }
                /*
                if writer.is_none() && p.filename().is_some() {
                    let origin_filename = dbg!(p.filename().unwrap());
                    let ext = get_ext(&origin_filename, allow_file_types)?;

                    match get_upload_file_writer(post_id, origin_filename, ext).await {
                        Ok((w, upload_file_info)) => {
                            writer = Some(w);
                            upload_info = Some(upload_file_info);
                        }
                        Err(e) => {
                            dbg!(e);
                            return Err(Error::UploadFailed);
                        }
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
                                }
                                None => {}
                            };
                        }
                        Err(e) => {
                            dbg!(e);
                            return Err(Error::UploadFailed);
                        }
                    };
                }
                */
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

    let mut vec: Vec<(BufWriter<File>, UploadFileInfo)> = file_writers.into_values().collect();
    while vec.len() > 0 {
        let mut entry = vec.swap_remove(0);
        entry.0.shutdown().await.map_err(|e| {
            dbg!(&e);
            eprintln!("{:?}", e);
            Error::UploadFailed
        })?;
        form_data_result.push(FormDataItem::FILE(entry.1));
    }

    Ok(form_data_result)
}

pub async fn save_upload_stream(
    post_id: u64,
    filename: String,
    mut body: impl Buf,
    allow_file_types: &[SupportFileType],
) -> Result<UploadFileInfo> {
    let mut upload_info: Option<UploadFileInfo> = None;

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

pub fn remove_dir(path: PathBuf) -> Result<()> {
    for result in std::fs::read_dir(path.as_path())? {
        let entry = result?;
        if let Ok(file_type) = entry.file_type() {
            // println!("{:?}: {:?}", entry.path(), file_type);
            if file_type.is_dir() {
                remove_dir(entry.path())?;
            } else if file_type.is_file() {
                // println!("std::fs::remove_file: {:?}", entry.path().as_path());
                let mut perms = std::fs::metadata(entry.path().as_path())?.permissions();
                if perms.readonly() {
                    perms.set_readonly(false);
                    std::fs::set_permissions(entry.path().as_path(), perms)?;
                }
                std::fs::remove_file(entry.path().as_path())?;
            }
        } else {
            println!("Couldn't get file type for {:?}", entry.path());
        }
    }
    // println!("std::fs::remove_dir: {:?}", path.as_path());
    std::fs::remove_dir(path.as_path())?;
    Ok(())
}
