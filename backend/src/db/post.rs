use core::time::Duration;
use std::time::SystemTime;

use comrak::{ComrakOptions, markdown_to_html};
use sqlx::{Sqlite, Row};

use blog_common::{
    dto::{
        post::{PostDetail, NewPost},
        PaginationData,
    },
    result::Error,
};

use crate::{
    db::{self, DATA_SOURCE, SqlParam, tag},
    util::snowflake,
};
use crate::db::model::{Post, Tag};
use crate::util::result::Result;

pub async fn list(page_num: u8, page_size: u8) -> Result<PaginationData<Vec<PostDetail>>> {
    let offset: i32 = ((page_num - 1) * page_size) as i32;
    let mut p: Vec<crate::db::SqlParam> = Vec::new();
    p.push(SqlParam::I32(offset));
    p.push(SqlParam::I8(page_size as i8));

    let id_array =
        db::sqlite_get_list::<crate::db::Id>("SELECT id FROM blog ORDER BY id DESC LIMIT ?,?", Some(p)).await?;
    let id_array: Vec<i64> = id_array.iter().map(|d| d.id).collect();

    let d = db::sled_get_list::<PostDetail>(&DATA_SOURCE.get().unwrap().blog, &id_array).await?;

    // let r = sqlx::query!("SELECT COUNT(id) AS total FROM blog").fetch_all(&DATASOURCE).await?;
    let row = sqlx::query("SELECT COUNT(id) FROM blog")
        .fetch_all(&DATA_SOURCE.get().unwrap().sqlite)
        // .fetch_one(&DATASOURCE.get().unwrap().sqlite)
        .await?;
    let total: i64 = row[0].get(0);
    // println!("total={}", total);

    Ok(PaginationData {
        total: total as u64,
        data: d,
    })
}

pub async fn list_by_tag(tag_name: String, page_num: u8, page_size: u8) -> Result<PaginationData<Vec<PostDetail>>> {
    let tag = sqlx::query_as::<Sqlite, Tag>("SELECT id FROM tag WHERE name = ?")
        .bind(&tag_name)
        .fetch_optional(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    if tag.is_none() {
        return Err(Error::SqliteDbError.into());
    }
    let tag = tag.unwrap();

    let r = sqlx::query("SELECT COUNT(*) FROM tag_usage WHERE tag_id = ?")
        .bind(tag.id)
        .fetch_one(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    let r = r.try_get::<i64, usize>(1);
    if let Err(e) = r {
        return Err(Error::SqliteDbError.into())
    }
    let total = r.unwrap() as u64;
    if total < 1 {
        return Ok(PaginationData {
            total,
            data: vec![],
        });
    }
    let d = sqlx::query_as::<Sqlite, Post>("SELECT p.* FROM post WHERE id IN (SELECT post_id FROM tag_usage WHERE tag_id = ? LIMIT ?, ?)")
        .bind(tag.id)
        .fetch_all(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    let d = d.iter().map(|i| i.into()).collect::<Vec<_>>();
    Ok(PaginationData {
        total,
        data: d,
    })
}

pub async fn save(new_post: NewPost) -> Result<PostDetail> {
    let now_sec = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => {
            eprintln!("{:?}", dbg!(e));
            return Err(Error::SaveBlogFailed.into());
        },
    };

    // todo needs to be in a transaction

    let id = snowflake::gen_id();
    // println!("id {}", id);
    if new_post.tags.is_some() {
        super::tag::record_usage(id, new_post.tags.unwrap()).await?;
    }

    // let parser = pulldown_cmark::Parser::new(body);
    // let mut html_text = String::new();
    // pulldown_cmark::html::push_html(&mut html_text, parser);

    let post = Post {
        id: id as i64,
        title: new_post.title,
        rendered_content: markdown_to_html(&new_post.content, &ComrakOptions::default()),
        markdown_content: new_post.content,
        created_at: now_sec,
        updated_at: None,
    };

    // save to sqlite
    let last_insert_rowid = sqlx::query(
        "INSERT INTO post(id, title, markdown_content, rendered_content, created_at)VALUES(?,?,?,?,?,?)",
    )
    .bind(&post.id)
    .bind(&post.title)
    .bind(&post.markdown_content)
    .bind(&post.rendered_content)
    .bind(&post.created_at)
    .execute(&DATA_SOURCE.get().unwrap().sqlite)
    .await?
    .last_insert_rowid();

    if last_insert_rowid < 1 {
        // println!("last_insert_rowid {}", last_insert_rowid);
        return Err(Error::SaveBlogFailed.into());
    }

    Ok((&post).into())
}

pub async fn show(id: u64) -> Result<PostDetail> {
    // let r: Option<PostDetail> = db::sled_get(&DATA_SOURCE.get().unwrap().blog, id.to_le_bytes()).await?;
    let r = sqlx::query_as::<Sqlite, Post>("SELECT * FROM post WHERE id = ?")
        .bind(id as i64)
        .fetch_optional(&DATA_SOURCE.get().unwrap().sqlite)
        .await?;
    if r.is_none() {
        Err(Error::CannotFoundBlog.into())
    } else {
        Ok((&r.unwrap()).into())
    }
}
