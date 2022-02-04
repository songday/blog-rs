use core::time::Duration;
use std::{collections::HashMap, time::SystemTime};

use blog_common::{
    dto::{
        post::{PostData, PostDetail},
        PaginationData,
    },
    result::Error,
    util::time,
    val,
};
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
    util::{
        common,
        result::{ErrorWrapper, Result},
        snowflake,
    },
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
    if posts.is_empty() {
        return Ok(vec![]);
    }
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

fn append_pagination_sql(sql: &mut String, pagination_type: &str, post_id: u64) -> bool {
    let mut order_by_asc = false;
    if post_id > 0 {
        let combine_word = if sql.rfind("WHERE").is_some() { "AND" } else { "WHERE" };
        sql.push_str(combine_word);
        if pagination_type == "prev" {
            sql.push_str(" id>");
            sql.push_str(post_id.to_string().as_str());
            order_by_asc = true;
        } else if pagination_type == "next" {
            sql.push_str(" id<");
            sql.push_str(post_id.to_string().as_str());
        }
    }
    sql.push_str(" ORDER BY id ");
    if order_by_asc {
        sql.push_str("ASC");
    } else {
        sql.push_str("DESC");
    }
    sql.push_str(" LIMIT ?");
    order_by_asc
}

pub async fn list(pagination_type: &str, post_id: u64, page_size: u8) -> Result<PaginationData<Vec<PostDetail>>> {
    let row = sqlx::query("SELECT COUNT(id) FROM post")
        .fetch_one(super::get_sqlite())
        .await?;
    let total: i64 = row.get(0);
    // println!("total={}", total);
    if total < 1 {
        return Ok(PaginationData { total: 0, data: vec![] });
    }

    let mut sql = String::with_capacity(256);
    sql.push_str(
        "SELECT id,title,title_image,'' AS markdown_content,'' AS rendered_content,created_at,updated_at FROM post ",
    );
    let order_by_asc = append_pagination_sql(&mut sql, pagination_type, post_id);
    println!("sql={}", sql);

    let mut d = sqlx::query_as::<Sqlite, Post>(&sql)
        .bind(page_size)
        .fetch_all(super::get_sqlite())
        .await?;
    if order_by_asc {
        d.reverse();
    }
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

pub async fn list_by_tag(
    tag_name: String,
    pagination_type: &str,
    post_id: u64,
    page_size: u8,
) -> Result<PaginationData<Vec<PostDetail>>> {
    let tag_name = urlencoding::decode(&tag_name)?;
    let s = tag_name.as_ref();
    let tag = sqlx::query_as::<Sqlite, Tag>("SELECT id,name FROM tag WHERE name = ?")
        .bind(s)
        .fetch_optional(super::get_sqlite())
        .await?;
    if tag.is_none() {
        return Err(Error::TagNotFound.into());
    }
    let tag = tag.unwrap();

    let r = sqlx::query("SELECT COUNT(*) FROM tag_usage WHERE tag_id = ?")
        .bind(tag.id)
        .fetch_one(super::get_sqlite())
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

    let mut sql = String::with_capacity(256);
    sql.push_str("SELECT id,title,title_image,'' AS markdown_content,rendered_content,created_at,updated_at FROM post WHERE id IN (SELECT post_id FROM tag_usage WHERE tag_id = ? ");
    let order_by_asc = append_pagination_sql(&mut sql, pagination_type, post_id);
    sql.push_str(")");
    println!("sql={}", sql);
    let mut d = sqlx::query_as::<Sqlite, Post>(
        // "SELECT id,title,title_image,'' AS markdown_content,rendered_content,created_at,updated_at FROM post WHERE id IN (SELECT post_id FROM tag_usage WHERE tag_id = ? ORDER BY id DESC LIMIT ?, ?)",
        &sql,
    )
    .bind(tag.id)
    .bind(page_size)
    .fetch_all(super::get_sqlite())
    .await?;
    if order_by_asc {
        d.reverse();
    }
    Ok(PaginationData {
        total: total as u64,
        data: to_detail_list(d).await?,
    })
}

pub async fn new_post() -> Result<i64> {
    let id = snowflake::gen_id() as i64;
    let last_insert_rowid =
        sqlx::query("INSERT INTO post(id, title, title_image, markdown_content, rendered_content, created_at)VALUES(?,?,'','','',?)")
            .bind(&id)
            .bind(val::DEFAULT_POST_TITLE)
            .bind(time::unix_epoch_sec() as i64)
            .execute(super::get_sqlite())
            .await?
            .last_insert_rowid();

    if last_insert_rowid < 1 {
        return Err(Error::SavePostFailed.into());
    }

    Ok(id)
}

pub async fn update_title_image(id: i64, title_image: &str) -> Result<()> {
    sqlx::query("UPDATE post SET title_image=? WHERE id=?")
        .bind(title_image)
        .bind(id)
        .execute(super::get_sqlite())
        .await?;

    Ok(())
}

async fn get_post(id: i64, edit: bool) -> Result<Option<Post>> {
    let sql = if edit {
        "SELECT id,title,title_image,'' AS markdown_content,markdown_content AS rendered_content,created_at,updated_at FROM post WHERE id = ?"
    } else {
        "SELECT id,title,title_image,'' AS markdown_content,rendered_content,created_at,updated_at FROM post WHERE id = ?"
    };
    sqlx::query_as::<Sqlite, Post>(sql)
        .bind(id)
        .fetch_optional(super::get_sqlite())
        .await
        .map_err(|e| {
            eprintln!("{:?}", e);
            Error::SqliteDbError.into()
        })
}

pub async fn save(post_data: PostData) -> Result<PostDetail> {
    let post = get_post(post_data.id, true).await?;
    if post.is_none() {
        return Err(Error::CannotFoundPost.into());
    }

    // needs to be in a transaction
    let transaction = super::get_sqlite().begin().await?;

    if post_data.tags.is_some() {
        super::tag::record_usage(post_data.id, post_data.tags.as_ref().unwrap()).await?;
    }

    // let parser = pulldown_cmark::Parser::new(body);
    // let mut html_text = String::new();
    // pulldown_cmark::html::push_html(&mut html_text, parser);

    let post = post.unwrap();

    let post_detail = PostDetail {
        id: post_data.id,
        title: post_data.title,
        title_image: post_data.title_image,
        content: markdown_to_html(&post_data.content, &ComrakOptions::default()),
        tags: post_data.tags,
        created_at: post.created_at as u64,
        updated_at: post.updated_at.map(|time| time as u64),
        editable: true,
    };

    let post_title = if post_detail.title.is_empty() {
        val::DEFAULT_POST_TITLE
    } else {
        &post_detail.title
    };

    // save to sqlite
    sqlx::query(
        "UPDATE post SET title=?, title_image=?, markdown_content=?, rendered_content=?, updated_at=? WHERE id=?",
    )
    .bind(post_title)
    .bind(&post_detail.title_image)
    .bind(&post_data.content)
    .bind(&post_detail.content)
    .bind(time::unix_epoch_sec() as i64)
    .bind(&post_detail.id)
    .execute(super::get_sqlite())
    .await?;

    // 这里只关心 commit，因为 https://docs.rs/sqlx/0.5.1/sqlx/struct.Transaction.html 说到
    // If neither are called before the transaction goes out-of-scope, rollback is called. In other words, rollback is called on drop if the transaction is still in-progress.
    transaction.commit().await?;

    Ok(post_detail)
}

pub async fn show(id: u64, editable: bool) -> Result<PostDetail> {
    // let r: Option<PostDetail> = db::sled_get(&DATA_SOURCE.get().unwrap().post, id.to_le_bytes()).await?;
    let id = id as i64;
    let r = get_post(id, editable).await?;
    if r.is_none() {
        Err(Error::CannotFoundPost.into())
    } else {
        let tags = sqlx::query_as::<Sqlite, Tag>("SELECT t.id AS id, t.name AS name FROM tag t INNER JOIN tag_usage u ON t.id = u.tag_id WHERE u.post_id = ? ORDER BY t.created_at DESC")
            .bind(id)
            .fetch_all(super::get_sqlite())
            .await?.iter().map(|t| t.name.clone()).collect();
        let mut post_detail: PostDetail = (&r.unwrap()).into();
        post_detail.tags = Some(tags);
        Ok(post_detail)
    }
}

pub async fn delete(id: u64) -> Result<()> {
    let r = sqlx::query("DELETE FROM post WHERE id=?")
        .bind(id as i64)
        .execute(super::get_sqlite())
        .await?;
    Ok(())
}

pub async fn all() -> Result<Vec<Post>> {
    let posts = sqlx::query_as::<Sqlite, Post>("SELECT * FROM post ORDER BY id DESC")
        .fetch_all(super::get_sqlite())
        .await?;
    Ok(posts)
}
