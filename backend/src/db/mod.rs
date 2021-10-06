use core::time::Duration;
use std::{
    io::ErrorKind,
    marker::{Send, Unpin},
    path::Path,
};

use blog_common::result::Error;
use futures::StreamExt;
use once_cell::sync::OnceCell;
use serde::Serialize;
use sqlx::{
    pool::PoolOptions,
    sqlite::{SqliteArguments, SqliteRow},
    Sqlite, SqlitePool,
};
use tokio::fs::{create_dir, remove_file, rename, File, OpenOptions};

use crate::util::result::Result;
use model::Tag;

pub(crate) mod management;
pub mod model;
pub(crate) mod post;
pub(crate) mod tag;
pub(crate) mod user;

type SqliteConnPool = sqlx::Pool<Sqlite>;

static DATA_SOURCE: OnceCell<DataSource> = OnceCell::new();

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
    // management: sled::Db,
    sqlite: SqliteConnPool,
}

#[derive(sqlx::FromRow)]
pub struct Id {
    id: i64,
}

pub fn get_sqlite() -> &SqliteConnPool {
    &DATA_SOURCE.get().unwrap().sqlite
}

pub async fn init_datasource() {
    let path = Path::new(".").join("blog.dat");
    if path.is_dir() {
        panic!("当前目录下有一个：blog.dat目录，请移动到另外一个地方再重试。");
    }
    let db_file_not_exists = !path.exists();
    if db_file_not_exists {
        let file = match OpenOptions::new()
            .read(false)
            .write(true)
            .create_new(true)
            .open(path.as_path())
            .await
        {
            Ok(f) => f,
            // Err(e: ErrorKind::NotFound) => None,
            Err(e) => panic!(e),
        };
    }
    let pool_ops = PoolOptions::<Sqlite>::new()
        .min_connections(8)
        .max_connections(64)
        .connect_timeout(Duration::from_secs(5))
        .test_before_acquire(true);
    let conn_str = format!("sqlite://{}", path.display());
    let pool = pool_ops
        .connect(conn_str.as_str())
        .await
        .expect("Init datasource failed.");

    if db_file_not_exists {
        // println!("Init database");
        let ddl = include_str!("../resource/sql/ddl.sql");
        // println!("ddl = {}", ddl);
        let mut stream = sqlx::query(ddl).execute_many(&pool).await;
        while let Some(res) = stream.next().await {
            match res {
                Ok(r) => println!("Initialized table"),
                Err(e) => panic!(e),
            }
        }
        let dml = include_str!("../resource/sql/dml.sql");
        if Err(e) = sqlx::query(dml).execute(&pool).await {
            panic!(e);
        }
    }

    let datasource = DataSource {
        sqlite: pool,
        // management: sled::open("data/management").expect("open"),
    };

    if let Err(e) = DATA_SOURCE.set(datasource) {
        panic!(e);
    }

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
    let ds = DATA_SOURCE.get().unwrap();
    ds.sqlite.close().await;
    // ds.management.flush();
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
    // let rows: Vec<Id> = sqlx::query_as!(Id, "SELECT id FROM post ORDER BY id").fetch_all(&d.sqlite).await?;
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
    let r: Vec<D> = q.fetch_all(&DATA_SOURCE.get().unwrap().sqlite).await?;
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
