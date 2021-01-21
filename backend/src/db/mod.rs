use core::time::Duration;
use std::marker::{Send, Unpin};

use once_cell::sync::OnceCell;
use serde::Serialize;
use sqlx::{
    pool::PoolOptions,
    sqlite::{SqliteArguments, SqliteRow},
    Sqlite, SqlitePool,
};

use blog_common::result::Error;

use crate::{model::Tag, result::Result};

pub(crate) mod blog;
pub(crate) mod tag;
pub(crate) mod user;

type SqliteConnPool = sqlx::Pool<Sqlite>;

static DATASOURCE: OnceCell<DataSource> = OnceCell::new();

// pub trait SqliteParam = for<'q> Encode<'q, Sqlite> + Type<Sqlite>;

pub enum SqlParam {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    STRING(String),
}

#[derive(Clone)]
pub struct DataSource {
    user: sled::Db,
    blog: sled::Db,
    sqlite: SqliteConnPool,
}

#[derive(sqlx::FromRow)]
pub struct Id {
    id: i64,
}

pub async fn init_datasource() {
    let pool_ops = PoolOptions::<Sqlite>::new()
        .min_connections(8)
        .max_connections(64)
        .connect_timeout(Duration::from_secs(2))
        .test_before_acquire(false);
    let pool = pool_ops
        .connect("sqlite://./data/all.db")
        .await
        .expect("Init datasource failed.");

    let datasource = DataSource {
        user: sled::open("data/user").expect("open"),
        blog: sled::open("data/blog").expect("open"),
        sqlite: pool,
    };

    if let Err(e) = DATASOURCE.set(datasource) {
        panic!(e);
    }

    match sqlite_get_list::<Tag>("SELECT name FROM blog_tag ORDER BY name ASC", None).await {
        Ok(tags) => {
            for tag in tags.iter() {
                tag::cache_tag_id(&tag.name);
            }
        },
        Err(e) => panic!(e),
    };

    /*
    下面这个不会打印，解决：
    1、把map换成for_each
    2、由于map是lazy的，所以需要在map后面加.collect()
     */
    /*
    match sqlite_get_list::<Tag>("SELECT id, name FROM blog_tag ORDER BY id DESC", None).await {
        Ok(tags) => tags.iter().map(|tag| {
            println!("{}", &tag.name);
            tag::put_id_name(tag.id, &tag.name);
        }),
        Err(e) => panic!(e),
    };
    */
}

pub async fn shutdown() {
    let ds = DATASOURCE.get().unwrap();
    ds.sqlite.close().await;
    ds.blog.flush();
    ds.user.flush();
}

#[inline]
pub(crate) async fn sled_save<D>(db: &sled::Db, key: impl AsRef<[u8]>, value: &D) -> Result<usize>
where
    D: Serialize,
{
    db.insert(key, serde_json::to_string(value).unwrap().as_str())?;
    db.flush_async().await.map_err(|e| {
        eprintln!("{}", e);
        Error::SledDbError.into()
    })
}

pub(crate) async fn sled_get<T>(db: &sled::Db, key: impl AsRef<[u8]>) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    if let Some(data) = db.get(key)? {
        let b: T = serde_json::from_slice(data.as_ref())?;
        return Ok(Some(b));
    }
    Ok(None)
}

pub(crate) async fn sled_get_list<T>(db: &sled::Db, id_array: &Vec<i64>) -> Result<Vec<T>>
where
    for<'d> T: serde::Deserialize<'d>,
{
    let mut list: Vec<T> = Vec::with_capacity(id_array.len());
    for id in id_array {
        if let Some(data) = db.get(id.to_le_bytes())? {
            let b: T = serde_json::from_slice(data.as_ref())?;
            list.push(b);
        }
    }
    Ok(list)
}

pub(crate) async fn sqlite_get_list<'a, D>(query: &'a str, params: Option<Vec<SqlParam>>) -> Result<Vec<D>>
where
    for<'r> D: sqlx::FromRow<'r, SqliteRow> + Send + Unpin,
    // P: SqliteParam,
    // for<'q> P: Encode<'q, Sqlite> + Type<Sqlite> + Send,
{
    // let rows: Vec<Id> = sqlx::query_as!(Id, "SELECT id FROM blog ORDER BY id").fetch_all(&d.sqlite).await?;
    // let mut conn = d.sqlite.acquire().await?;
    let mut q = sqlx::query_as::<Sqlite, D>(query);
    if let Some(params) = params {
        for p in params {
            match p {
                SqlParam::I8(v) => q = q.bind(v),
                SqlParam::I16(v) => q = q.bind(v),
                SqlParam::I32(v) => q = q.bind(v),
                SqlParam::I64(v) => q = q.bind(v),
                SqlParam::STRING(v) => q = q.bind(v),
            };
        }
    }
    let r: Vec<D> = q.fetch_all(&DATASOURCE.get().unwrap().sqlite).await?;
    // let d = sqlx::query_as::<Sqlite, D>(query)
    //     .bind(0i32)
    //     .bind(crate::vars::BLOG_PAGE_SIZE)
    //     .fetch_all(&self.sqlite)
    //     .await?;
    Ok(r)
}

// https://github.com/wyanlord/rust-web-demo/blob/master/src/dao/mod.rs

#[macro_export]
macro_rules! sql_query_one (
    ($sql: expr, $($bind: expr),*) => ({
        let pool = match db::mysql::get_pool() {
            Some(p) => p,
            None => return Err("mysql get pool failed".into()),
        };

        match sqlx::query_as(&$sql)$(.bind($bind))*.fetch_one(pool).await {
            Ok(u) => Ok(Some(u)),
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(None),
                _ => Err(e.into())
            },
        }
    });
    ($sql: expr) => (query_one!($sql,));
);
