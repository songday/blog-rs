use std::rc::Rc;
use std::vec::Vec;

use blog_common::dto::post::PostDetail;
use blog_common::dto::{PaginationData, Response};
use blog_common::val;
use weblog::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::component::PostsListComponent;
use crate::router::Route;

pub enum Msg {
    Compose,
}

pub struct PostsList {}

impl Component for PostsList {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Compose => {
                let navigator = ctx.link().navigator().unwrap();
                wasm_bindgen_futures::spawn_local(async move {
                    let response = reqwasm::http::Request::get("/post/new").send().await.unwrap();
                    let json: Response<u64> = response.json().await.unwrap();
                    if json.status == 0 {
                        navigator.push(crate::router::Route::ComposePost { id: json.data.unwrap() });
                        // yew_router::push_route(crate::router::Route::ComposePost { id: json.data.unwrap() });
                    } else {
                        // ctx.link().location().unwrap().route().set_href("/management");
                        if let Some(loc) = web_sys::window().map(|window| window.location()) {
                            let _ = loc.set_href("/management");
                        } else {
                            console_log!("get location failed");
                        }
                    }
                });
            },
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        gloo::utils::document().set_title("博客/Blog");

        html! {
            <>
                <div class="columns">
                    <div class="column is-10">
                        <h1 class="title is-1">{ "博客/Posts" }</h1>
                        <h2 class="subtitle">{ "All of your quality writing in one place" }</h2>
                    </div>
                </div>
                <PostsListComponent request_uri={"/post/list/".to_string()} />
            </>
        }
    }
}
