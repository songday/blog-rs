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

pub(super) async fn record_usage(post_id: u64, tags: Vec<String>) -> Result<()> {
    Ok(())
}