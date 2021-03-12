use std::{
    collections::HashMap,
    hash::Hasher,
    io::{Cursor, ErrorKind, SeekFrom},
    mem::size_of,
    path::{Path, PathBuf},
    sync::Arc,
    vec::Vec,
};

use ahash::AHasher;
use bytes::{Buf, Bytes, BytesMut};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use tokio::{
    fs::{File, OpenOptions, remove_file, rename},
    // io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    io::{self, AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter},
};
use sqlx::Sqlite;

use blog_common::result::Error;

use crate::db::model::Tag;
use crate::{
    db::{self, DATA_SOURCE},
    util::{crypt, snowflake},
};
use crate::util::result::Result;

pub async fn list() -> Result<Vec<String>> {
    let tag_list = sqlx::query_as::<Sqlite, Tag>("SELECT name FROM tag ORDER BY created_at DESC")
        .fetch_all(&DATA_SOURCE.get().unwrap().sqlite).await?;
    let name_list = tag_list.iter().map(|i| i.name.clone()).collect::<Vec<String>>();
    Ok(name_list)
}

pub async fn get_names(id_array: Vec<i64>) -> Result<Vec<String>> {
    if id_array.is_empty() {
        return Ok(vec![]);
    }
    let mut sql = String::with_capacity(256);
    sql.push_str("SELECT name from tag WHERE id IN (");
    for id in id_array.iter() {
        sql.push_str(id.to_string().as_str());
        sql.push(',');
    }
    sql.push('-');
    sql.replace(",-", ") ORDER BY created_at DESC");
    let tag_list = sqlx::query_as::<Sqlite, Tag>(sql.as_str())
        .fetch_all(&DATA_SOURCE.get().unwrap().sqlite).await?;
    let name_list = tag_list.iter().map(|i| i.name.clone()).collect::<Vec<String>>();
    Ok(name_list)
}

pub(super) async fn record_usage(post_id: u64, tags: &Vec<String>) -> Result<()> {
    // query id list by name list
    let mut sql = String::with_capacity(256);
    sql.push_str("SELECT id from tag WHERE name IN (");
    for _i in 0..tags.len() {
        sql.push_str("?,");
    }
    sql.replace_range(sql.len().., ")");
    let mut query = sqlx::query_as::<Sqlite, Tag>(sql.as_str());
    for tag in tags.iter() {
        query = query.bind(tag);
    }
    let tags = query.fetch_all(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;

    let post_id = post_id as i64;
    for tag in tags {
        sqlx::query("INSERT INTO tag_usage(post_id, tag_id)VALUES(?,?)").bind(post_id).bind(tag.id).execute(&DATA_SOURCE.get().unwrap().sqlite).await?;
    }
    Ok(())
}