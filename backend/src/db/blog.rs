use core::time::Duration;
use std::time::SystemTime;

use comrak::{markdown_to_html, ComrakOptions};
use sqlx::Row;

use blog_common::{
    dto::{
        blog::{BlogDetail, NewBlog},
        PaginationData,
    },
    result::Error,
};

use crate::{
    db::{self, tag, SqlParam, DATASOURCE},
    model::{Blog, Tag},
    result::Result,
    util::snowflake,
    var,
};

pub async fn list(page_num: u8, page_size: u8) -> Result<PaginationData<Vec<BlogDetail>>> {
    let offset: i32 = ((page_num - 1) * page_size) as i32;
    let mut p: Vec<crate::db::SqlParam> = Vec::new();
    p.push(SqlParam::I32(offset));
    p.push(SqlParam::I8(page_size as i8));

    let id_array =
        db::sqlite_get_list::<crate::db::Id>("SELECT id FROM blog ORDER BY id DESC LIMIT ?,?", Some(p)).await?;
    let id_array: Vec<i64> = id_array.iter().map(|d| d.id).collect();

    let d = db::sled_get_list::<BlogDetail>(&DATASOURCE.get().unwrap().blog, &id_array).await?;

    // let r = sqlx::query!("SELECT COUNT(id) AS total FROM blog").fetch_all(&DATASOURCE).await?;
    let row = sqlx::query("SELECT COUNT(id) FROM blog")
        .fetch_all(&DATASOURCE.get().unwrap().sqlite)
        // .fetch_one(&DATASOURCE.get().unwrap().sqlite)
        .await?;
    let total: i64 = row[0].get(0);
    // println!("total={}", total);

    Ok(PaginationData {
        total: total as u64,
        data: d,
    })
}

pub fn tags() -> Result<Vec<Tag>> { Ok(tag::get_tags()) }

pub async fn list_by_tag(tag: String, page_num: u8, page_size: u8) -> Result<PaginationData<Vec<BlogDetail>>> {
    let blog_data = tag::get_blog_id_data(&tag);
    // let mut id_array = blog_data.id_array.to_owned();
    let id_array = blog_data.get_id_array(page_num, page_size).await?;
    // id_array.reverse();
    // let mut end = offset + var::BLOG_PAGE_SIZE;
    // if end > id_array.len() {
    //     end = id_array.len();
    // }
    let d = db::sled_get_list::<BlogDetail>(&DATASOURCE.get().unwrap().blog, &id_array.id_array).await?;

    Ok(PaginationData {
        total: id_array.total,
        data: d,
    })
}

async fn save_tags(mut new_blog: NewBlog) -> Result<(NewBlog, Option<Vec<Tag>>, Option<String>)> {
    if new_blog.tags.is_none() || new_blog.tags.as_ref().unwrap().len() == 0 {
        return Ok((new_blog, None, None));
    }

    let tags = new_blog.tags.as_mut().unwrap();
    let tags_amount = tags.len();
    if tags_amount == 0 {
        return Ok((new_blog, None, None));
    }

    let mut all_tags_str = String::with_capacity(512);
    let mut sql = String::with_capacity(128);
    let mut params: Vec<SqlParam> = Vec::with_capacity(tags_amount);

    sql.push_str("SELECT name from blog_tag WHERE name IN (");
    for tag in tags.iter_mut() {
        let s = tag.trim().to_string();
        *tag = s.clone();
        all_tags_str.push_str(tag);
        all_tags_str.push('\n');
        sql.push_str("?,");
        params.push(SqlParam::STRING(s));
    }
    let l = sql.len();
    sql.insert(l - 1, ')');
    sql.remove(l);
    all_tags_str.remove(all_tags_str.len() - 1);

    let mut tags_in_db = db::sqlite_get_list::<Tag>(sql.as_str(), Some(params)).await?;

    for tag_name in tags.iter() {
        match tags_in_db.binary_search_by(|s| s.name.cmp(tag_name)) {
            Ok(_i) => {
                // tags_in_db.remove(i);
            },
            _ => {
                let new_tag = Tag {
                    name: tag_name.replace("/", "、").replace(r"\", "、"),
                };
                let _r = sqlx::query("INSERT INTO blog_tag(name)VALUES(?)")
                    .bind(&new_tag.name)
                    .execute(&DATASOURCE.get().unwrap().sqlite)
                    .await?;
                crate::db::tag::cache_tag_id(&new_tag.name);
                tags_in_db.push(new_tag);
            },
        };
    }

    Ok((new_blog, Some(tags_in_db), Some(all_tags_str)))
}

pub async fn save(new_blog: NewBlog) -> Result<BlogDetail> {
    let now_sec = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => {
            eprintln!("{:?}", dbg!(e));
            return Err(Error::SaveBlogFailed.into());
        },
    };

    let (new_blog, all_tags, all_tags_string) = save_tags(new_blog).await?;

    let id = snowflake::gen_id() as i64;
    // println!("id {}", id);

    // let parser = pulldown_cmark::Parser::new(body);
    // let mut html_text = String::new();
    // pulldown_cmark::html::push_html(&mut html_text, parser);

    let blog = Blog {
        id,
        title: new_blog.title,
        parsed_content: markdown_to_html(&new_blog.content, &ComrakOptions::default()),
        markdown_content: new_blog.content,
        tags: {
            if all_tags_string.is_none() {
                String::new()
            } else {
                all_tags_string.unwrap()
            }
        },
        created_at: now_sec,
        updated_at: None,
    };

    // save to sqlite
    let last_insert_rowid = sqlx::query(
        "INSERT INTO blog(id, title, markdown_content, parsed_content, tags, created_at)VALUES(?,?,?,?,?,?)",
    )
    .bind(&blog.id)
    .bind(&blog.title)
    .bind(&blog.markdown_content)
    .bind(&blog.parsed_content)
    .bind(&blog.tags)
    .bind(&blog.created_at)
    .execute(&DATASOURCE.get().unwrap().sqlite)
    .await?
    .last_insert_rowid();

    if last_insert_rowid < 1 {
        // println!("last_insert_rowid {}", last_insert_rowid);
        return Err(Error::SaveBlogFailed.into());
    }

    if all_tags.is_some() {
        for tag in all_tags.unwrap().iter() {
            // println!("save blog id for tag:{}", tag.name);
            let mut blog_id_data = tag::get_blog_id_data(&tag.name);
            blog_id_data.add_id(id);
            blog_id_data.save_to_disk().await?;
        }
    }

    // save to sled
    // let b: Option<Blog> = sled_get(d, 100).await?;
    let key = blog.id.to_le_bytes();
    let blog_detail = blog.into();
    let cnt = db::sled_save(&DATASOURCE.get().unwrap().blog, key, &blog_detail).await?;
    if cnt > 0 {
        Ok(blog_detail)
    } else {
        Err(Error::SaveBlogFailed.into())
    }
}

pub async fn show(id: u64) -> Result<BlogDetail> {
    let r: Option<BlogDetail> = db::sled_get(&DATASOURCE.get().unwrap().blog, id.to_le_bytes()).await?;
    if r.is_none() {
        Err(Error::CannotFoundBlog.into())
    } else {
        Ok(r.unwrap())
    }
}
