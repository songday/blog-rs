use std::{collections::HashMap, convert::Infallible, net::SocketAddr};

use futures::future::Future;
use tokio::sync::oneshot::Receiver;
use warp::{self, reject, Filter};

use blog_common::{
    dto::{
        management::{AdminUser, Settings},
        post::PostData,
        user::UserInfo,
    },
    val,
};

use crate::{
    facade::{self, asset, image, management, post, tag, user},
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

// pub async fn create_server(
//     address: &str,
//     receiver: Receiver<()>,
// ) -> result::Result<(impl Future<Output = ()> + 'static)> {
//     let datasource = Arc::new(crate::db::get_datasource().await?);
//     let (_addr, server) = create_warp_server(address, datasource, receiver)?;
//     Ok(server)
// }

pub async fn create_warp_server(address: &str, receiver: Receiver<()>) -> Result<impl Future<Output = ()> + 'static> {
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
        .and(warp::body::json::<Settings>())
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
    let post_list = warp::get()
        .and(warp::path("post"))
        .and(warp::path("list"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<u64>())
        .and(warp::path::end())
        .and_then(post::list);
    let tag_list = warp::get()
        .and(warp::path("tag"))
        .and(warp::path("list"))
        .and(warp::path::end())
        .and_then(tag::list);
    let top_tags = warp::get()
        .and(warp::path("tag"))
        .and(warp::path("top"))
        .and(warp::path::end())
        .and_then(tag::top);
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

    let cors = warp::cors()
        // .allow_any_origin()
        .allow_origins(
            vec![
                "http://localhost:9270",
                "http://127.0.0.1:9270",
                // todo 读取配置里面的域名信息，然后填写在这里
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
        .or(tag_list)
        .or(top_tags)
        .or(post_list_by_tag)
        .or(post_new)
        .or(post_save)
        .or(post_delete)
        .or(post_show)
        .or(upload_image)
        .or(upload_title_image)
        .or(save_image)
        .with(cors)
        // .with(warp::service(session_id_wrapper))
        .recover(facade::handle_rejection);

    let addr = address.parse::<SocketAddr>()?;

    let (_addr, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, async {
        receiver.await.ok();
    });

    Ok(server)
}
