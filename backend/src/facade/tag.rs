use core::{convert::Infallible, result::Result};

use warp::{Rejection, Reply};

use crate::{
    db::tag,
    facade::{session_id_cookie, wrap_json_data, wrap_json_err},
};

pub async fn list() -> Result<impl Reply, Rejection> {
    match tag::list().await {
        Ok(list) => Ok(wrap_json_data(&list)),
        Err(e) => Ok(wrap_json_err(500, e.0)),
    }
}
