use std::vec::Vec;
use std::{collections::HashMap, convert::Infallible, net::SocketAddr};

use futures::future::Future;
use hyper::{header::HeaderValue, HeaderMap, Uri};
use password_hash::Output;
use tokio::sync::oneshot::Receiver;
use warp::{self, reject, Filter, Rejection, Reply, Server, TlsServer};

use blog_common::{
    dto::{
        management::{AdminUser, Setting},
        post::PostData,
        user::UserInfo,
    },
    val,
};

use crate::{
    facade::{self, asset, export, image, management, post, tag, user},
    service::status,
    util::result::Result,
};

#[derive(Debug)]
struct FilterError;

impl reject::Reject for FilterError {}

// https://github.com/seanmonstar/warp/issues/16
// https://github.com/jxs/warp/blob/add-wrap-fn/examples/wrapping.rs#L4
fn session_id_wrapper<F, T>(filter: F) -> impl Filter<Extract = (&'static str,)> + Clone + Send + Sync + 'static
where
    F: Filter<Extract = (T,), Error = Infallible> + Clone + Send + Sync + 'static,
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
    warp::cookie::optional(val::SESSION_ID_HEADER_NAME).map(|a: Option<String>| match a {
        Some(s) => match status::check_auth(Some(s)) {
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

fn hsts_header_appender<F, T>(
    filter: F,
) -> impl Filter<Extract = (warp::reply::WithHeader<T>,)> + Clone + Send + Sync + 'static
where
    T: warp::Reply,
    F: Filter<Extract = (T,), Error = std::convert::Infallible> + Clone + Send + Sync + 'static,
    F::Extract: warp::Reply,
{
    warp::any()
        // .map(|| {
        //     println!("before filter");
        // })
        // .untuple_one()
        .and(filter)
        .map(|reply| {
            warp::reply::with_header(
                reply,
                "Strict-Transport-Security",
                "max-age=31536000; includeSubDomains; preload",
            )
        })
}

// pub async fn create_server(
//     address: &str,
//     receiver: Receiver<()>,
// ) -> result::Result<(impl Future<Output = ()> + 'static)> {
//     let datasource = Arc::new(crate::db::get_datasource().await?);
//     let (_addr, server) = create_warp_server(address, datasource, receiver)?;
//     Ok(server)
// }

pub async fn create_static_file_server(
    address: SocketAddr,
    receiver: Receiver<()>,
) -> Result<impl Future<Output = ()> + 'static> {
    let dir = std::env::current_dir().unwrap();

    println!("Serving directory path is {}", dir.as_path().display());
    let routes = warp::get().and(warp::fs::dir(dir));

    //let addr = address.parse::<SocketAddr>()?;

    let (_addr, server) = warp::serve(routes).bind_with_graceful_shutdown(address, async {
        receiver.await.ok();
    });

    Ok(server)
}

pub async fn create_blog_server(
    http_addr: SocketAddr,
    receiver: Receiver<()>,
    cors_host: &Option<String>,
) -> Result<impl Future<Output = ()> + 'static> {
    let routes = blog_filter("http", http_addr.port(), cors_host);
    // let routes = routes.recover(facade::handle_rejection);
    let server = warp::serve(routes);
    let server = server
        .bind_with_graceful_shutdown(http_addr, async {
            receiver.await.ok();
        })
        .1;
    return Ok(server);
}

// enum MyServer {
//     normal(Server<dyn Filter<Extract = dyn Reply, Error = Rejection, Future = dyn Future<Output = core::result::Result<dyn Reply, Rejection>> + Send>>),
//     https(TlsServer<dyn Filter<Extract = dyn Reply, Error = Rejection, Future = dyn Future<Output = core::result::Result<dyn Reply, Rejection>> + Send>>),
// }
//
// fn get_server() -> Option<Server<impl Filter<Extract = impl warp::Reply, Error = warp::Rejection>>> {
//     None
// }

pub async fn create_tls_blog_server(
    https_addr: SocketAddr,
    receiver: Receiver<()>,
    cert_path: &str,
    key_path: &str,
    cors_host: &Option<String>,
) -> Result<impl Future<Output = ()> + 'static> {
    let routes = blog_filter("https", https_addr.port(), cors_host);
    // let routes = routes.recover(facade::handle_rejection);
    let server = warp::serve(routes);
    let server = server.tls().cert_path(cert_path).key_path(key_path);
    let server = server
        .bind_with_graceful_shutdown(https_addr, async {
            receiver.await.ok();
        })
        .1;
    return Ok(server);
}

pub async fn create_tls_blog_server_with_hsts(
    https_addr: SocketAddr,
    receiver: Receiver<()>,
    cert_path: &str,
    key_path: &str,
    cors_host: &Option<String>,
) -> Result<impl Future<Output = ()> + 'static> {
    let routes = blog_filter("https", https_addr.port(), cors_host);
    // let routes = routes.recover(facade::handle_rejection);
    let routes = routes.with(warp::wrap_fn(hsts_header_appender));
    let server = warp::serve(routes);
    let server = server.tls().cert_path(cert_path).key_path(key_path);
    let server = server
        .bind_with_graceful_shutdown(https_addr, async {
            receiver.await.ok();
        })
        .1;
    return Ok(server);
}

pub fn blog_filter(
    scheme: &str,
    port: u16,
    cors_host: &Option<String>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = core::convert::Infallible> + Clone {
    // pub fn blog_filter(scheme: &str, port: u16, cors_host: &Option<String>,) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let index = warp::get().and(warp::path::end()).and_then(crate::facade::index::index);
    let asset = warp::get()
        .and(warp::path("asset"))
        .and(warp::path::tail())
        .and(warp::path::end())
        .and_then(asset::get_asset);
    let get_upload = warp::get()
        .and(warp::path("upload"))
        .and(warp::path::tail())
        .and(warp::path::end())
        .and_then(image::get_upload_image);
    let management_settings = warp::get()
        .and(warp::path("management"))
        .and(warp::path::end())
        .and(warp::cookie::optional(val::SESSION_ID_HEADER_NAME))
        .and_then(management::index);
    let management_login = warp::post()
        .and(warp::path("management"))
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(warp::cookie::optional(val::SESSION_ID_HEADER_NAME))
        .and(warp::body::json::<AdminUser>())
        .and_then(management::admin_login);
    let management_update_settings = warp::post()
        .and(warp::path("management"))
        .and(warp::path("settings"))
        .and(warp::path("update"))
        .and(warp::path::end())
        .and(warp::cookie::optional(val::SESSION_ID_HEADER_NAME))
        .and(warp::body::json::<Setting>())
        .and_then(management::update_settings);
    let user_logout = warp::get()
        .and(warp::path("user"))
        .and(warp::path("logout"))
        .and(warp::path::end())
        .and(warp::cookie::optional(val::SESSION_ID_HEADER_NAME))
        .and_then(user::logout);
    let user_info = warp::get()
        .and(warp::path("user"))
        .and(warp::path("info"))
        .and(warp::path::end())
        .and(warp::cookie::optional(val::SESSION_ID_HEADER_NAME))
        .and_then(user::info);
    let verify_image = warp::get()
        .and(warp::path("tool"))
        .and(warp::path("verify-image"))
        .and(warp::path::end())
        .and(warp::cookie::optional(val::SESSION_ID_HEADER_NAME))
        .and_then(image::verify_image);
    let random_title_image = warp::get()
        .and(warp::path("tool"))
        .and(warp::path("random-title-image"))
        .and(warp::path::param::<u64>())
        .and(warp::path::end())
        .and_then(image::random_title_image);
    let tags_all = warp::get()
        .and(warp::path("tags"))
        .and(warp::path("all"))
        .and(warp::path::end())
        .and_then(tag::list);
    let top_tags = warp::get()
        .and(warp::path("tag"))
        .and(warp::path("top"))
        .and(warp::path::end())
        .and_then(tag::top);
    let post_list = warp::get()
        .and(warp::path("post"))
        .and(warp::path("list"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<u64>())
        .and(warp::path::end())
        .and_then(post::list);
    let post_list_by_tag = warp::get()
        .and(warp::path("post"))
        .and(warp::path("tag"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::param::<u64>())
        .and(warp::path::end())
        .and_then(post::list_by_tag);
    let post_new = warp::get()
        .and(warp::path("post"))
        .and(warp::path("new"))
        .and(warp::path::end())
        .and(warp::cookie::optional(val::SESSION_ID_HEADER_NAME))
        .and_then(post::new);
    let post_save = warp::post()
        .and(warp::path("post"))
        .and(warp::path("save"))
        .and(warp::path::end())
        .and(auth())
        .and(warp::body::json::<PostData>())
        .and_then(post::save);
    let post_delete = warp::get()
        .and(warp::path("post"))
        .and(warp::path("delete"))
        .and(warp::path::param::<u64>())
        .and(warp::path::end())
        .and(auth())
        .and_then(post::delete);
    let post_show = warp::get()
        .and(warp::path("post"))
        .and(warp::path("show"))
        .and(warp::cookie::optional(val::SESSION_ID_HEADER_NAME))
        .and(warp::path::param::<u64>())
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::path::end())
        .and_then(post::show);
    let upload_image = warp::post()
        .and(warp::path("image"))
        .and(warp::path("upload"))
        .and(warp::path::param::<u64>())
        .and(warp::path::end())
        .and(auth())
        .and(warp::multipart::form().max_length(val::MAX_BLOG_UPLOAD_IMAGE_SIZE as u64))
        .and_then(image::upload);
    let upload_title_image = warp::post()
        .and(warp::path("image"))
        .and(warp::path("upload-title-image"))
        .and(warp::path::param::<u64>())
        .and(warp::path::end())
        .and(auth())
        .and(warp::multipart::form().max_length(val::MAX_BLOG_UPLOAD_IMAGE_SIZE as u64))
        .and_then(image::upload_title_image);
    let save_image = warp::post()
        .and(warp::path("image"))
        .and(warp::path("save"))
        .and(warp::path::param::<u64>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::body::content_length_limit(val::MAX_BLOG_UPLOAD_IMAGE_SIZE as u64))
        .and(auth())
        .and(warp::body::aggregate())
        .and_then(image::save);
    let export = warp::get()
        .and(warp::path("export"))
        .and(warp::path::tail())
        .and(warp::path::end())
        .and(auth())
        .and_then(export::export_handler);
    let forgot_password = warp::get()
        .and(warp::path("management"))
        .and(warp::path("forgot-password"))
        .and(warp::path::end())
        .and(warp::host::optional())
        .and_then(management::forgot_password);

    // Setting CORS
    let mut host = String::with_capacity(32);
    let mut origins: Vec<&str> = Vec::with_capacity(5);

    host.push_str(scheme);
    host.push_str("://localhost");
    if port != 80 && port != 443 {
        host.push_str(":");
        host.push_str(&port.to_string());
    }
    origins.push(&host);
    let host = host.replace("localhost", "127.0.0.1");
    origins.push(&host);
    if cors_host.is_some() {
        origins.push(&cors_host.as_ref().unwrap());
    }

    let cors = warp::cors()
        // .allow_any_origin()
        .allow_origins(origins)
        .max_age(60 * 60)
        // 当需要 Fetch 传 Cookie 的时候，需要下面这行
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials
        .allow_credentials(true)
        .allow_headers(vec!["Authorization", "Content-Type"].into_iter())
        .allow_methods(vec!["GET", "POST", "DELETE"].into_iter())
        .build();
    // End

    // Combine routes
    let routes = index
        .or(asset)
        .or(get_upload)
        .or(management_settings)
        .or(management_login)
        .or(management_update_settings)
        .or(user_logout)
        .or(user_info)
        .or(verify_image)
        .or(random_title_image)
        .or(post_list)
        .or(tags_all)
        .or(top_tags)
        .or(post_list_by_tag)
        .or(post_new)
        .or(post_save)
        .or(post_delete)
        .or(post_show)
        .or(upload_image)
        .or(upload_title_image)
        .or(save_image)
        .or(export)
        .or(forgot_password)
        .with(cors);
    // End

    // let t:() = routes;
    // let t:() = routes.recover(facade::handle_rejection);

    // routes
    routes.recover(facade::handle_rejection)
}
