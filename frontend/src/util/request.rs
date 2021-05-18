use alloc::{format, vec::Vec};
use core::fmt::Debug;

use blog_common::{dto::Response as ApiResponse, val};
use serde::{Deserialize, Serialize};
use yew::{
    callback::Callback,
    format::{Binary, Json, Nothing, Text},
    services::{
        console::ConsoleService,
        fetch::{FetchOptions, FetchService, FetchTask, Request, Response},
    },
};

use crate::util::{store, Error};

pub(crate) enum RequestMethod {
    GET,
    POST,
}

fn get_url(uri: &str) -> String {
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

pub(crate) fn text_callback_handler<T>(callback: Callback<Result<T, Error>>) -> impl Fn(Response<Text>) -> ()
where
    for<'de> T: Deserialize<'de> + 'static + Debug,
{
    move |response: Response<Text>| {
        ConsoleService::log("收到回调");
        let (meta, data) = response.into_parts();
        let data = match data {
            Ok(d) => d,
            Err(e) => {
                ConsoleService::log(&format!("收到错误回调 {}", e));
                callback.emit(Err(Error::RequestError));
                return;
            },
        };
        if meta.status.is_success() {
            ConsoleService::log("收到回调 success");
            let data: Result<ApiResponse<T>, _> = serde_json::from_str(&data);
            match data {
                Ok(d) => {
                    ConsoleService::log("收到返回");
                    if d.status == 0 {
                        ConsoleService::log("返回的是成功");
                        callback.emit(Ok(d.data.unwrap()))
                    } else {
                        ConsoleService::log("返回的是失败");
                        let detail = d.error.unwrap().detail;
                        ConsoleService::log(&detail);
                        callback.emit(Err(Error::BusinessError(detail)))
                    }
                },
                Err(e) => callback.emit(Err(Error::DeserializeError)),
            }
        } else {
            let status = meta.status.as_u16();
            ConsoleService::log(&format!("收到回调 failed status: {}", status));
            match status {
                401 => callback.emit(Err(Error::Unauthorized)),
                403 => callback.emit(Err(Error::Forbidden)),
                404 => callback.emit(Err(Error::NotFound)),
                500 => callback.emit(Err(Error::InternalServerError)),
                // 422 => {
                //     let data: Result<ErrorInfo, _> = serde_json::from_str(&data);
                //     if let Ok(data) = data {
                //         callback.emit(Err(Error::UnprocessableEntity(data)))
                //     } else {
                //         callback.emit(Err(Error::DeserializeError))
                //     }
                // }
                _ => callback.emit(Err(Error::RequestError)),
            }
        }
    }
}

fn fetch_options() -> FetchOptions {
    // 根据：https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API 知道，
    // 2017/8月之前的浏览器，Fetch默认不发送Cookie，需要增加下面的 option 才可以
    FetchOptions {
        // credentials: Some(web_sys::RequestCredentials::SameOrigin),
        // For CORS requests, use credentials: 'include' to allow sending credentials to other domains:
        // https://github.com/github/fetch#sending-cookies
        credentials: Some(web_sys::RequestCredentials::Include),
        // mode: Some(web_sys::RequestMode::Navigate),
        ..FetchOptions::default()
    }
}

pub(crate) fn get<T>(uri: &str, callback: Callback<Result<T, Error>>) -> FetchTask
where
    for<'de> T: Deserialize<'de> + 'static + Debug,
{
    create_fetch_task(RequestMethod::GET, uri, Nothing, callback)
}

pub(crate) fn post<B, T>(uri: &str, body: B, callback: Callback<Result<T, Error>>) -> FetchTask
where
    for<'de> T: Deserialize<'de> + 'static + Debug,
    B: Serialize,
{
    let body: Text = Json(&body).into();
    create_fetch_task(RequestMethod::POST, uri, body, callback)
}

pub(crate) fn create_fetch_task<B, T>(
    method: RequestMethod,
    uri: &str,
    body: B,
    callback: Callback<Result<T, Error>>,
) -> FetchTask
where
    for<'de> T: Deserialize<'de> + 'static + Debug,
    B: Into<Text>,
{
    /*
    let handler = move |response: Response<Text>| {
        if let (meta, Ok(data)) = response.into_parts() {
            ConsoleService::log("收到回调");
            if meta.status.is_success() {
                ConsoleService::log("收到回调 success");
                let data: Result<ApiResponse<T>, _> = serde_json::from_str(&data);
                match data {
                    Ok(d) => {
                        ConsoleService::log("收到返回");
                        if d.status == 0 {
                            ConsoleService::log("返回的是成功");
                            callback.emit(Ok(d.data.unwrap()))
                        } else {
                            ConsoleService::log("返回的是失败");
                            let detail = d.error.unwrap().detail;
                            ConsoleService::log(&detail);
                            callback.emit(Err(Error::BusinessError(detail)))
                        }
                    },
                    Err(e) => {
                        callback.emit(Err(Error::DeserializeError))
                    },
                }
            } else {
                ConsoleService::log("收到回调 failed 1");
                match meta.status.as_u16() {
                    401 => callback.emit(Err(Error::Unauthorized)),
                    403 => callback.emit(Err(Error::Forbidden)),
                    404 => callback.emit(Err(Error::NotFound)),
                    500 => callback.emit(Err(Error::InternalServerError)),
                    // 422 => {
                    //     let data: Result<ErrorInfo, _> = serde_json::from_str(&data);
                    //     if let Ok(data) = data {
                    //         callback.emit(Err(Error::UnprocessableEntity(data)))
                    //     } else {
                    //         callback.emit(Err(Error::DeserializeError))
                    //     }
                    // }
                    _ => callback.emit(Err(Error::RequestError)),
                }
            }
        } else {
            ConsoleService::log("收到回调 failed 1");
            callback.emit(Err(Error::RequestError))
        }
    };
    */

    let url = get_url(uri);
    let mut request = match method {
        RequestMethod::GET => Request::get(url),
        RequestMethod::POST => Request::post(url),
    };
    request = request.header("Content-Type", "application/json");
    if let Some(token) = store::get(val::SESSION_ID_HEADER_NAME) {
        request = request.header("Authorization", token);
    }

    let request = request.body(body).unwrap();

    let handler = text_callback_handler(callback);

    FetchService::fetch_with_options(request, fetch_options(), handler.into()).unwrap()
}

pub fn post_binary<T>(uri: &str, body: Vec<u8>, callback: Callback<Result<T, Error>>) -> FetchTask
where
    for<'de> T: Deserialize<'de> + 'static + Debug,
{
    let callback = move |response: Response<Binary>| {
        let (meta, data) = response.into_parts();
        let data = match data {
            Ok(d) => d,
            Err(e) => {
                ConsoleService::log(&format!("收到错误回调1 {}", e));
                callback.emit(Err(Error::RequestError));
                return;
            },
        };
        println!("META: {:?}, {:?}", meta, data);
        if meta.status.is_success() {
            let data = serde_json::from_slice::<ApiResponse<T>>(data.as_slice());
            match data {
                Ok(d) => {
                    ConsoleService::log("收到返回");
                    if d.status == 0 {
                        ConsoleService::log("返回的是成功");
                        callback.emit(Ok(d.data.unwrap()))
                    } else {
                        ConsoleService::log("返回的是失败");
                        let detail = d.error.unwrap().detail;
                        ConsoleService::log(&detail);
                        callback.emit(Err(Error::BusinessError(detail)))
                    }
                },
                Err(e) => callback.emit(Err(Error::DeserializeError)),
            }
        } else {
            let status = meta.status.as_u16();
            ConsoleService::log(&format!("收到回调 failed status: {}", status));
            match status {
                401 => callback.emit(Err(Error::Unauthorized)),
                403 => callback.emit(Err(Error::Forbidden)),
                404 => callback.emit(Err(Error::NotFound)),
                500 => callback.emit(Err(Error::InternalServerError)),
                // 422 => {
                //     let data: Result<ErrorInfo, _> = serde_json::from_str(&data);
                //     if let Ok(data) = data {
                //         callback.emit(Err(Error::UnprocessableEntity(data)))
                //     } else {
                //         callback.emit(Err(Error::DeserializeError))
                //     }
                // }
                _ => callback.emit(Err(Error::RequestError)),
            }
        }
    };
    let builder = Request::post(uri);
    let request = builder.body(Ok(body)).unwrap();
    FetchService::fetch_binary_with_options(request, fetch_options(), callback.into()).unwrap()
}
