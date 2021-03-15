use core::{convert::Infallible, result::Result};

use warp::{Rejection, Reply};

use crate::{
    db::tag,
    facade::{auth_cookie, response_data, response_err},
};

pub async fn list() -> Result<impl Reply, Rejection> {
    match tag::list().await {
        Ok(list) => Ok(response_data(&list)),
        Err(e) => Ok(response_err(500, e.0)),
    }
}
