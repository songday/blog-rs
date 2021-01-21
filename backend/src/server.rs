use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use futures::future::Future;
use tokio::sync::oneshot::Receiver;

use warp::{self, reject, Filter, Rejection, Server};

use blog_common::{
    dto::{
        blog::NewBlog,
        user::{LoginParams, RegisterParams, UserInfo},
    },
    var,
};

use crate::{controller, db::DataSource, result::Result, service::status};

#[derive(Debug)]
struct FilterError;

impl reject::Reject for FilterError {}

// https://github.com/seanmonstar/warp/issues/16
// https://github.com/jxs/warp/blob/add-wrap-fn/examples/wrapping.rs#L4
fn session_id_wrapper<F, T>(filter: F) -> impl Filter<Extract = (&'static str,)> + Clone + Send + Sync + 'static
where
    F: Filter<Extract = (T,), Error = std::convert::Infallible> + Clone + Send + Sync + 'static,
    F::Extract: warp::Reply,
{
    warp::any()
        .map(|| {
            println!("before filter");
        })
        .untuple_one()
        .and(filter)
        .map(|_arg| "wrapped hello world")
}

// https://github.com/seanmonstar/warp/issues/177#issuecomment-469497434
// https://stackoverflow.com/questions/54988438/how-to-check-the-authorization-header-using-warp

fn auth() -> impl Filter<Extract = (Option<UserInfo>,), Error = Infallible> + Clone {
    warp::cookie::optional(var::AUTH_HEADER_NAME).map(|a: Option<String>| match a {
        Some(s) => match status::check_auth(&s) {
            Ok(u) => Some(u),
            Err(_) => None,
        },
        None => None,
    })
    // warp::header::<String>("x-auth").and_then(|token: String| async move {
    //     status::check_auth(&token).map_err(|e| {
    //         eprintln!("{:?}", e);
    //         reject::custom(FilterError)
    //     })
    // })
}

// pub async fn create_server(
//     address: &str,
//     receiver: Receiver<()>,
// ) -> result::Result<(impl Future<Output = ()> + 'static)> {
//     let datasource = Arc::new(crate::db::get_datasource().await?);
//     let (_addr, server) = create_warp_server(address, datasource, receiver)?;
//     Ok(server)
// }

pub async fn create_warp_server(address: &str, receiver: Receiver<()>) -> Result<impl Future<Output = ()> + 'static> {
    let index = warp::get().and(warp::path::end()).and_then(controller::index);
    let about = warp::get()
        .and(warp::path("about"))
        .and(warp::path::end())
        .and(warp::get())
        .and_then(controller::about);
    let user_login = warp::post()
        .and(warp::path("user"))
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(warp::cookie::optional(var::AUTH_HEADER_NAME))
        .and(warp::body::json::<LoginParams>())
        .and_then(controller::user_login);
    let user_register = warp::post()
        .and(warp::path("user"))
        .and(warp::path("register"))
        .and(warp::path::end())
        .and(warp::cookie::optional(var::AUTH_HEADER_NAME))
        .and(warp::body::json::<RegisterParams>())
        .and_then(controller::user_register);
    let user_logout = warp::get()
        .and(warp::path("user"))
        .and(warp::path("logout"))
        .and(warp::path::end())
        .and(warp::cookie::optional(var::AUTH_HEADER_NAME))
        .and_then(controller::user_logout);
    let user_info = warp::get()
        .and(warp::path("user"))
        .and(warp::path("info"))
        .and(warp::path::end())
        .and(warp::cookie::optional(var::AUTH_HEADER_NAME))
        .and_then(controller::user_info);
    let verify_image = warp::get()
        .and(warp::path("tool"))
        .and(warp::path("verify-image"))
        .and(warp::path::end())
        .and(warp::cookie::optional(var::AUTH_HEADER_NAME))
        .and_then(controller::verify_image);
    let blog_list = warp::get()
        .and(warp::path("blog"))
        .and(warp::path("list"))
        .and(warp::path::param::<u8>())
        .and(warp::path::end())
        .and_then(controller::blog_list);
    let blog_tags = warp::get()
        .and(warp::path("blog"))
        .and(warp::path("tags"))
        .and(warp::path::end())
        .and_then(controller::blog_tags);
    let blog_list_by_tag = warp::get()
        .and(warp::path("blog"))
        .and(warp::path("tag"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<u8>())
        .and(warp::path::end())
        .and_then(controller::blog_list_by_tag);
    let blog_save = warp::post()
        .and(warp::path("blog"))
        .and(warp::path("save"))
        .and(warp::path::end())
        .and(auth())
        .and(warp::body::json::<NewBlog>())
        .and_then(controller::blog_save);
    let blog_show = warp::get()
        .and(warp::path("blog"))
        .and(warp::path("show"))
        .and(warp::path::param::<u64>())
        .and(warp::path::end())
        .and_then(controller::blog_show);
    let blog_upload_image = warp::post()
        .and(warp::path("blog"))
        .and(warp::path("image"))
        .and(warp::path("upload"))
        .and(warp::path::end())
        .and(auth())
        .and(warp::multipart::form().max_length(var::MAX_BLOG_UPLOAD_IMAGE_SIZE as u64))
        .and_then(controller::blog_upload_image);
    let blog_save_image = warp::post()
        .and(warp::path("blog"))
        .and(warp::path("image"))
        .and(warp::path("save"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::content_length_limit(var::MAX_BLOG_UPLOAD_IMAGE_SIZE as u64))
        .and(auth())
        .and(warp::body::aggregate())
        .and_then(controller::blog_save_image);

    let cors = warp::cors()
        // .allow_any_origin()
        .allow_origins(
            vec![
                "http://localhost:8080",
                "http://localhost:9270",
                "http://127.0.0.1:8080",
                "http://127.0.0.1:9270",
                "http://www.songday.com",
                "https://www.songday.com",
            ]
            .into_iter(),
        )
        .max_age(60 * 60)
        // 当需要 Fetch 传 Cookie 的时候，需要下面这行
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials
        .allow_credentials(true)
        .allow_headers(vec!["Authorization", "Content-Type"].into_iter())
        .allow_methods(vec!["GET", "POST", "DELETE"].into_iter())
        .build();

    let routes = index
        .or(about)
        .or(user_login)
        .or(user_register)
        .or(user_logout)
        .or(user_info)
        .or(verify_image)
        .or(blog_list)
        .or(blog_tags)
        .or(blog_list_by_tag)
        .or(blog_save)
        .or(blog_show)
        .or(blog_upload_image)
        .or(blog_save_image)
        .with(cors)
        // .with(warp::service(session_id_wrapper))
        .recover(controller::handle_rejection);

    let addr = address.parse::<SocketAddr>()?;

    let (_addr, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, async {
        receiver.await.ok();
    });

    Ok(server)
}
