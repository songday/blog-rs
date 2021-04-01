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
use blog_common::result::Error;
use bytes::{Buf, Bytes, BytesMut};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use sqlx::Sqlite;
use tokio::{
    fs::{remove_file, rename, File, OpenOptions},
    io::{self, AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter},
};

use crate::{
    db::{self, model::Tag, DATA_SOURCE},
    util::{common, crypt, result::Result, snowflake},
};

pub async fn list() -> Result<Vec<String>> {
    let tag_list = sqlx::query_as::<Sqlite, Tag>("SELECT name FROM tag ORDER BY created_at DESC")
        .fetch_all(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
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
        .fetch_all(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    let name_list = tag_list.iter().map(|i| i.name.clone()).collect::<Vec<String>>();
    Ok(name_list)
}

pub(super) async fn record_usage(post_id: u64, tags: &Vec<String>) -> Result<()> {
    // query id list by name list
    let mut sql = String::with_capacity(256);
    sql.push_str("SELECT id,name from tag WHERE name IN (");
    for _i in 0..tags.len() {
        sql.push_str("?,");
    }
    sql.replace_range(sql.len() - 1.., ")");
    // println!("{}", sql.as_str());
    let mut query = sqlx::query_as::<Sqlite, Tag>(sql.as_str());
    for tag in tags.iter() {
        query = query.bind(tag);
    }
    let mut tags_in_db = query.fetch_all(&DATA_SOURCE.get().unwrap().sqlite).await?;

    // 查看有没有新的tag
    if tags_in_db.len() < tags.len() {
        let mut new_tags: Vec<Tag> = Vec::with_capacity(tags.len() - tags_in_db.len());
        {
            let mut tags_in_db_iter = tags_in_db.iter();
            for tag in tags.iter() {
                if !tags_in_db_iter.any(|e| e.name.eq(tag)) {
                    let id = sqlx::query("INSERT INTO tag(name, created_at)VALUES(?,?)")
                        .bind(tag)
                        .bind(common::get_current_sec()? as i64)
                        .execute(&DATA_SOURCE.get().unwrap().sqlite)
                        .await?
                        .last_insert_rowid();
                    let new_tag = Tag {
                        id,
                        name: String::from(tag),
                    };
                    new_tags.push(new_tag);
                }
            }
        }
        tags_in_db.append(&mut new_tags);
    }

    let post_id = post_id as i64;
    for tag in tags_in_db {
        sqlx::query("INSERT INTO tag_usage(post_id, tag_id)VALUES(?,?)")
            .bind(post_id)
            .bind(tag.id)
            .execute(&DATA_SOURCE.get().unwrap().sqlite)
            .await?;
    }
    Ok(())
}
