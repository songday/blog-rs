use std::{
    collections::HashMap,
    hash::Hasher,
    io::{Cursor, ErrorKind, SeekFrom},
    mem::size_of,
    path::{Path, PathBuf},
    sync::Arc,
};

use ahash::AHasher;
use bytes::{Buf, Bytes, BytesMut};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use tokio::{
    fs::{remove_file, rename, File, OpenOptions},
    // io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    io::{self, AsyncReadExt, AsyncWriteExt, AsyncSeekExt, BufReader, BufWriter},
};

use blog_common::result::Error;

use crate::{model::Tag, result::Result, var};

pub struct BlogIdData {
    tag: String,
    filepath: PathBuf,
    pub new_id_array: Vec<i64>,
}

pub struct BlogIdArray {
    pub total: u64,
    pub id_array: Vec<i64>,
}

type TagNameIdMap = HashMap<String, u64>;

lazy_static! {
    static ref TAGS_MAP: Arc<RwLock<TagNameIdMap>> = Arc::new(RwLock::new(HashMap::with_capacity(64)));
}

pub fn get_tags() -> Vec<Tag> {
    let d = TAGS_MAP.read();
    let mut tags: Vec<Tag> = Vec::with_capacity(d.len());
    for (name, _id) in d.iter() {
        tags.push(Tag {
            name: String::from(name),
        });
    }
    tags
}

pub fn cache_tag_id(name: &str) -> u64 {
    let mut hasher = AHasher::default();
    hasher.write(name.as_bytes());
    let id = hasher.finish();
    TAGS_MAP.write().insert(String::from(name), id);
    id
}

pub fn get_id_by_name(name: &str) -> u64 {
    {
        let d = TAGS_MAP.read();
        if let Some(id) = d.get(name) {
            return *id;
        }
    }
    cache_tag_id(name)
}

pub fn get_blog_id_data(name: &str) -> BlogIdData {
    let mut p = PathBuf::with_capacity(128);
    p.push("data");
    p.push("tag");
    p.push(get_id_by_name(name).to_string().as_str());
    p.set_extension("dat");

    BlogIdData {
        tag: String::from(name),
        filepath: p,
        new_id_array: Vec::with_capacity(8),
    }
}

impl BlogIdData {
    pub async fn get_id_array(&self, page_num: u8, page_size: u8) -> Result<BlogIdArray> {
        let file = match OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(dbg!(self.filepath.as_path()))
            .await
        {
            Ok(f) => Some(f),
            // Err(e: ErrorKind::NotFound) => None,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => None,
                _ => {
                    eprintln!("here 1 {:?}", e);
                    return Err(Error::ReadBlogIdDataByTagFailed.into());
                },
            },
        };

        if file.is_none() {
            return Ok(BlogIdArray {
                total: 0,
                id_array: Vec::new(),
            });
        }

        let mut file = file.unwrap();
        let metadata = file.metadata().await?;
        let filesize = metadata.len();

        let offset = (page_num as u64 - 1) * page_size as u64;
        let offset_bytes = offset * var::I64SIZE as u64;
        if offset_bytes > filesize {
            return Ok(BlogIdArray {
                total: 0,
                id_array: Vec::new(),
            });
        }

        if offset_bytes > 0 {
            file.seek(SeekFrom::Start(offset_bytes)).await?;
        }

        let mut buffer = BytesMut::with_capacity(page_size as usize * var::I64SIZE);

        let mut r = BufReader::<File>::with_capacity(32768, file);
        let read_amount = r.read_buf(&mut buffer).await?;

        let len = (read_amount / var::I64SIZE) as usize;
        let mut d = BlogIdArray {
            total: filesize / var::I64SIZE as u64,
            id_array: Vec::with_capacity(len),
        };

        let b = buffer.freeze();
        // b.into_vec();
        let mut buf_reader = Cursor::new(b);

        for _idx in 0..len {
            match buf_reader.read_i64().await {
                Ok(i) => {
                    println!("id={}", i);
                    d.id_array.push(i);
                },
                Err(e) => {
                    dbg!(e);
                    break;
                },
            };
        }
        Ok(d)
    }

    pub fn add_id(&mut self, id: i64) { self.new_id_array.push(id); }

    pub async fn remove_id(&mut self, id: i64) -> Result<()> {
        let mut id_array: Vec<i64> = Vec::with_capacity(256);

        {
            let file = OpenOptions::new()
                .read(true)
                .write(false)
                .append(false)
                .create(false)
                .open(self.filepath.as_path())
                .await?;
            let mut r = BufReader::<File>::with_capacity(32768, file);
            loop {
                match r.read_i64().await {
                    Ok(i) => id_array.push(i),
                    Err(e) => {
                        dbg!(e);
                        break;
                    },
                };
            }
        }

        match id_array.binary_search(&id) {
            Ok(i) => {
                id_array.remove(i);
            },
            _ => {},
        }

        let file = OpenOptions::new()
            .read(false)
            .write(true)
            .append(false)
            .create(false)
            .truncate(true)
            .open(self.filepath.as_path())
            .await?;

        let mut w = BufWriter::<File>::with_capacity(32768, file);
        for id in id_array.iter() {
            w.write_i64(*id);
        }

        Ok(())
    }

    pub async fn save_to_disk(&self) -> Result<()> {
        if self.new_id_array.is_empty() {
            return Ok(());
        }

        let mut new_filename = PathBuf::with_capacity(128);
        new_filename.push("data");
        new_filename.push("tag");
        new_filename.push(self.tag.as_str());
        new_filename.set_extension("temporary");
        let new_file = OpenOptions::new()
            .read(false)
            .write(true)
            .append(false)
            .create(true)
            .open(dbg!(new_filename.as_path()))
            .await?;

        let mut w = BufWriter::<File>::with_capacity(10240, new_file);
        for i in self.new_id_array.iter() {
            w.write_i64(*i).await?;
        }

        let original_filepath = self.filepath.as_path();
        if let Ok(original_file) = OpenOptions::new()
            .read(true)
            .write(false)
            .append(false)
            .create(false)
            .open(dbg!(original_filepath))
            .await
        {
            let mut buffer = BytesMut::with_capacity(10240);
            let mut r = BufReader::<File>::with_capacity(10240, original_file);
            loop {
                let read_amount = r.read_buf(&mut buffer).await?;
                if read_amount < 1 {
                    break;
                }
            }
            println!("buffer size={}", buffer.len());
            while buffer.has_remaining() {
                w.write_buf(&mut buffer).await?;
            }
            remove_file(original_filepath).await?;
        }

        w.shutdown().await?;

        rename(new_filename.as_path().to_str().unwrap(), original_filepath).await?;

        Ok(())
    }
}
