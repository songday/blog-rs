use core::time::Duration;
use std::{collections::HashMap, time::SystemTime};

use blog_common::{
    dto::{
        post::{PostData, PostDetail},
        PaginationData,
    },
    result::Error,
};
use chrono::{Timelike, TimeZone, Utc};
// use chrono::offset::Utc;
// use chrono::DateTime;
use comrak::{markdown_to_html, ComrakOptions};
use sqlx::{Row, Sqlite};

use crate::{
    db::{
        self,
        model::{Post, Tag},
        tag,
        tag::get_names,
        SqlParam, DATA_SOURCE,
    },
    util::{common, result::{ErrorWrapper, Result}, snowflake},
};

// const START_TIME: DateTime<Utc> = Utc.ymd(1970, 1, 1).and_hms(0, 1, 1);

fn review_rendered_content(c: &str) -> String {
    let r = common::HTML_TAG_REGEX.replace_all(c, "");
    let r = r.replace(r"\n", "").to_string();
    if r.len() < 200 {
        return r;
    }
    return r[..200].to_string();
}

async fn to_detail_list(posts: Vec<Post>) -> Result<Vec<PostDetail>> {
    let post_ids: Vec<i64> = posts.iter().map(|p| p.id).collect();
    let tags_map = tag::get_tags_by_post_ids(post_ids).await?;
    let post_detail_list = posts
        .iter()
        .map(|i| {
            let mut detail: PostDetail = i.into();
            detail.content = review_rendered_content(&i.rendered_content);
            let tags = tags_map.get(&i.id);
            if tags.is_some() {
                detail.tags = Some(tags.unwrap().iter().map(|t| t.name.clone()).collect());
            }
            detail
        })
        .collect::<Vec<_>>();
    // for post in post_detail_list.iter_mut() {
    //     let tags = tags_map.get(&post.id);
    // if tags.is_none() {
    //     continue;
    // }
    //     *post.tags = Some(tags.unwrap().iter().map(|t| t.name.clone()).collect());
    // }
    Ok(post_detail_list)
}

pub async fn list(page_num: u8, page_size: u8) -> Result<PaginationData<Vec<PostDetail>>> {
    let row = sqlx::query("SELECT COUNT(id) FROM post")
        .fetch_one(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    let total: i64 = row.get(0);
    // println!("total={}", total);
    if total < 1 {
        return Ok(PaginationData { total: 0, data: vec![] });
    }

    let mut offset: i64 = ((page_num - 1) * page_size) as i64;
    if offset > total {
        offset = total - page_size as i64;
    }
    let d = sqlx::query_as::<Sqlite, Post>(
        "SELECT id,title,'' AS markdown_content,rendered_content,created_at,updated_at FROM post WHERE title!='' ORDER BY id DESC LIMIT ?, ?",
    )
        .bind(offset as i64)
        .bind(page_size)
        .fetch_all(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    Ok(PaginationData {
        total: total as u64,
        data: to_detail_list(d).await?,
    })
    /*
    let mut p: Vec<crate::db::SqlParam> = Vec::new();
    p.push(SqlParam::I64(offset));
    p.push(SqlParam::I8(page_size as i8));
    let id_array =
        db::sqlite_get_list::<crate::db::Id>("SELECT id FROM post ORDER BY id DESC LIMIT ?,?", Some(p)).await?;
    let id_array: Vec<i64> = id_array.iter().map(|d| d.id).collect();
    let d = db::sled_get_list::<PostDetail>(&DATA_SOURCE.get().unwrap().setting, &id_array).await?;
    */
}

pub async fn list_by_tag(tag_name: String, page_num: u8, page_size: u8) -> Result<PaginationData<Vec<PostDetail>>> {
    let tag_name = urlencoding::decode(&tag_name)?;
    let tag = sqlx::query_as::<Sqlite, Tag>("SELECT id,name FROM tag WHERE name = ?")
        .bind(&tag_name)
        .fetch_optional(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    if tag.is_none() {
        return Err(Error::TagNotFound.into());
    }
    let tag = tag.unwrap();

    let r = sqlx::query("SELECT COUNT(*) FROM tag_usage WHERE tag_id = ?")
        .bind(tag.id)
        .fetch_one(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    let r = r.try_get::<i64, usize>(0);
    if let Err(e) = r {
        eprintln!("{:?}", e);
        return Err(Error::SqliteDbError.into());
    }

    let total = r.unwrap();
    if total < 1 {
        return Ok(PaginationData { total: 0, data: vec![] });
    }

    let mut offset: i64 = ((page_num - 1) * page_size) as i64;
    if offset > total {
        offset = total - page_size as i64;
    }
    let d = sqlx::query_as::<Sqlite, Post>(
        "SELECT id,title,'' AS markdown_content,rendered_content,created_at,updated_at FROM post WHERE id IN (SELECT post_id FROM tag_usage WHERE tag_id = ? ORDER BY id DESC LIMIT ?, ?)",
    )
    .bind(tag.id)
    .bind(offset as i64)
    .bind(page_size)
    .fetch_all(&DATA_SOURCE.get().unwrap().sqlite)
    .await?;
    Ok(PaginationData {
        total: total as u64,
        data: to_detail_list(d).await?,
    })
}

pub async fn new_post() -> Result<i64> {
    let id = snowflake::gen_id() as i64;
    let last_insert_rowid =
        sqlx::query("INSERT INTO post(id, title, markdown_content, rendered_content, created_at)VALUES(?,'','','',?)")
            .bind(&id)
            .bind(chrono::offset::Utc::now().timestamp() as i64)
            .execute(super::get_sqlite())
            .await?
            .last_insert_rowid();

    if last_insert_rowid < 1 {
        return Err(Error::SavePostFailed.into());
    }

    Ok(id)
}

async fn get_post(id: i64, edit: bool) -> Result<Option<Post>> {
    let sql = if edit {
            "SELECT id,title,'' AS markdown_content,markdown_content AS rendered_content,created_at,updated_at FROM post WHERE id = ?"
        } else {
            "SELECT id,title,'' AS markdown_content,rendered_content,created_at,updated_at FROM post WHERE id = ?"
        };
    sqlx::query_as::<Sqlite, Post>(sql)
        .bind(id)
        .fetch_optional(super::get_sqlite())
        .await.map_err(|e| {eprintln!("{:?}", e);Error::SqliteDbError.into()})
}

pub async fn save(post_data: PostData) -> Result<PostDetail> {
    let post = get_post(post_data.id, true).await?;
    if post.is_none() {
        return Err(Error::CannotFoundPost.into());
    }

    // needs to be in a transaction
    let transaction = DATA_SOURCE.get().unwrap().sqlite.begin().await?;

    if post_data.tags.is_some() {
        super::tag::record_usage(post_data.id, post_data.tags.as_ref().unwrap()).await?;
    }

    // let parser = pulldown_cmark::Parser::new(body);
    // let mut html_text = String::new();
    // pulldown_cmark::html::push_html(&mut html_text, parser);

    let post_detail = PostDetail {
        id: post_data.id,
        title: post_data.title,
        content: markdown_to_html(&post_data.content, &ComrakOptions::default()),
        tags: post_data.tags,
        created_at: Utc.timestamp(post.unwrap().created_at, 0),
        updated_at: chrono::offset::Utc::now(),
        editable: true,
    };

    // save to sqlite
    let last_insert_rowid =
        sqlx::query("UPDATE post SET title=?, markdown_content=?, rendered_content=?, updated_at=? WHERE id=?")
            .bind(&post_detail.title)
            .bind(&post_data.content)
            .bind(&post_detail.content)
            .bind(post_detail.updated_at.timestamp() as i64)
            .bind(&post_detail.id)
            .execute(&DATA_SOURCE.get().unwrap().sqlite)
            .await?
            .last_insert_rowid();

    if last_insert_rowid < 1 {
        // println!("last_insert_rowid {}", last_insert_rowid);
        return Err(Error::SavePostFailed.into());
    }

    // 这里只关心 commit，因为 https://docs.rs/sqlx/0.5.1/sqlx/struct.Transaction.html 说到
    // If neither are called before the transaction goes out-of-scope, rollback is called. In other words, rollback is called on drop if the transaction is still in-progress.
    transaction.commit().await?;

    Ok(post_detail)
}

pub async fn show(id: u64, query_string: HashMap<String, String>) -> Result<PostDetail> {
    // let r: Option<PostDetail> = db::sled_get(&DATA_SOURCE.get().unwrap().post, id.to_le_bytes()).await?;
    let id = id as i64;
    let r = get_post(id, query_string.contains_key("edit")).await?;
    if r.is_none() {
        Err(Error::CannotFoundPost.into())
    } else {
        let tags = sqlx::query_as::<Sqlite, Tag>("SELECT t.id AS id, t.name AS name FROM tag t INNER JOIN tag_usage u ON t.id = u.tag_id WHERE u.post_id = ? ORDER BY t.created_at DESC")
            .bind(id)
            .fetch_all(&DATA_SOURCE.get().unwrap().sqlite)
            .await?.iter().map(|t| t.name.clone()).collect();
        let mut post_detail: PostDetail = (&r.unwrap()).into();
        post_detail.tags = Some(tags);
        Ok(post_detail)
    }
}
