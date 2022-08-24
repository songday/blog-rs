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
use blog_common::{dto::tag::TagUsageAmount, result::Error, util::time};
use bytes::{Buf, Bytes, BytesMut};
use parking_lot::RwLock;
use sqlx::{Row, Sqlite};
use tokio::{
    fs::{remove_file, rename, File, OpenOptions},
    io::{self, AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader, BufWriter},
};

use crate::{
    db::{self, model::Tag, DATA_SOURCE},
    util::{common, crypt, result::Result, snowflake},
};

pub async fn top() -> Result<Vec<TagUsageAmount>> {
    let tags = sqlx::query("SELECT t.id,t.name,u.amount FROM tags t INNER JOIN (SELECT tag_id, COUNT(tag_id) AS amount FROM tags_usage GROUP BY tag_id) u ON t.id=u.tag_id ORDER BY u.amount DESC")
        .fetch_all(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    let name_list = tags
        .iter()
        .map(|i| TagUsageAmount {
            id: i.get(0),
            name: i.get(1),
            amount: i.get(2),
        })
        .collect::<Vec<TagUsageAmount>>();
    Ok(name_list)
}

pub async fn list() -> Result<Vec<String>> {
    let tag_list = sqlx::query_as::<Sqlite, Tag>("SELECT id,name FROM tags ORDER BY created_at DESC")
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
    sql.push_str("SELECT name from tags WHERE id IN (");
    for id in id_array.iter() {
        sql.push_str(id.to_string().as_str());
        sql.push(',');
    }
    sql.push('-');
    let sql = sql.replace(",-", ") ORDER BY created_at DESC");
    let tag_list = sqlx::query_as::<Sqlite, Tag>(sql.as_str())
        .fetch_all(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    let name_list = tag_list.iter().map(|i| i.name.clone()).collect::<Vec<String>>();
    Ok(name_list)
}

pub(super) async fn record_usage(post_id: i64, tags: &Vec<String>) -> Result<()> {
    // query id list by name list
    let mut sql = String::with_capacity(256);
    sql.push_str("SELECT id,name from tags WHERE name IN (");
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
                    let id = sqlx::query("REPLACE INTO tags(name, created_at)VALUES(?,?)")
                        .bind(tag)
                        .bind(time::unix_epoch_sec() as i64)
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

    // 把没有用到的tag id删除
    if tags_in_db.len() > 0 {
        let mut sql = String::with_capacity(512);
        sql.push_str("DELETE FROM tags_usage WHERE post_id = ? AND tag_id NOT IN (");
        for _idx in 0..tags_in_db.len() {
            sql.push_str("?,");
        }
        sql.replace_range(sql.len() - 1.., ")");
        // println!("{}", sql.as_str());
        let mut query = sqlx::query(sql.as_str());
        for tag in tags_in_db.iter() {
            query = query.bind(tag.id);
        }
        let _tags_in_db = query.execute(&DATA_SOURCE.get().unwrap().sqlite).await?;
    }

    let post_id = post_id;
    for tag in tags_in_db {
        sqlx::query("REPLACE INTO tags_usage(post_id, tag_id)VALUES(?,?)")
            .bind(post_id)
            .bind(tag.id)
            .execute(&DATA_SOURCE.get().unwrap().sqlite)
            .await?;
    }
    Ok(())
}

pub(crate) async fn get_tags_by_post_ids(ids: Vec<i64>) -> Result<HashMap<i64, Vec<Tag>>> {
    let mut sql = String::from(
        "SELECT u.post_id, t.id, t.name FROM tags_usage u INNER JOIN tags t ON u.tag_id = t.id WHERE u.post_id IN (",
    );
    for _i in 0..ids.len() {
        sql.push_str("?,");
    }
    sql.replace_range(sql.len() - 1.., ")");
    let mut query = sqlx::query(&sql);
    for id in ids.iter() {
        query = query.bind(id);
    }
    let r = query.fetch_all(&DATA_SOURCE.get().unwrap().sqlite).await?;
    let mut d: HashMap<i64, Vec<Tag>> = HashMap::with_capacity(ids.len());

    for row in r {
        let tags = d.entry(row.get(0)).or_insert(vec![]);
        tags.push(Tag {
            id: row.get(1),
            name: row.get(2),
        });
    }

    Ok(d)
}
