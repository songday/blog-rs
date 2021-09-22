use alloc::{format, vec::Vec};
use core::fmt::Debug;

use blog_common::{dto::Response as ApiResponse, val, result::Result};
use serde::{Deserialize, Serialize};
use yew::{
    callback::Callback,
    format::{Binary, Json, Nothing, Text},
};
use reqwasm::http::{Request, Response};

use crate::util::{store, Error};

pub(crate) enum RequestMethod {
    GET,
    POST,
}

pub async fn get<D: Deserialize>(uri: &str) -> Result<D>
    where
            for<'de> D: Deserialize<'de> + 'static + Debug,
{
    let url = construct_url_from_uri(uri);
    let resp = Request::get(&url).send().await?;
    resp.json()?
}

fn construct_url_from_uri(uri: &str) -> String {
    let location = web_sys::window().unwrap().location();
    let mut prefix = String::with_capacity(32);
    if let Ok(p) = location.protocol() {
        prefix.push_str(&p);
    }
    prefix.push_str("//");
    if let Ok(h) = location.hostname() {
        prefix.push_str(&h);
    }
    if let Ok(p) = location.port() {
        prefix.push(':');
        prefix.push_str(&p);
    }
    prefix.push_str("/");
    prefix.push_str(uri);
    prefix
}
