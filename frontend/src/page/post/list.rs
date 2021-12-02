use std::vec::Vec;

use blog_common::dto::post::PostDetail;
use blog_common::dto::Response;
use weblog::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;

pub enum Msg {
    Compose,
    Retrieve,
}

pub struct PostList {
    posts: Vec<PostDetail>,
}

impl Component for PostList {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            posts: vec![],
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Compose => {
                let history = ctx.link().history().unwrap();
                wasm_bindgen_futures::spawn_local(async move {
                    let response = reqwasm::http::Request::get("/post/new").send().await.unwrap();
                    let json: Response<u64> = response.json().await.unwrap();
                    if json.status == 0 {
                        history.push(crate::router::Route::ComposePost { id: json.data.unwrap() });
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
            }
            Msg::Retrieve => {
                let posts = use_state(|| vec![]);
                {
                    let posts = posts.clone();
                    use_effect_with_deps(
                        move |_| {
                            let posts = posts.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                let response: Response<Vec<PostDetail>> = reqwasm::http::Request::get("/post/list/1")
                                    .send()
                                    .await
                                    .unwrap()
                                    .json()
                                    .await
                                    .unwrap();
                                posts.set(response.data.unwrap());
                            });
                            || ()
                        },
                        (),
                    );
                }
                self.posts = (*posts).clone();        
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // let page = self.current_page();
        let row_num = self.posts.len() / 2 + 1;
        let mut left_column_data: Vec<&PostDetail> = Vec::with_capacity(row_num);
        let mut right_column_data: Vec<&PostDetail> = Vec::with_capacity(row_num);
        let mut is_odd = true;
        for post in self.posts.iter() {
            if is_odd {
                left_column_data.push(post);
                is_odd = false;
            } else {
                right_column_data.push(post);
                is_odd = true;
            }
        }
        html! {
            <>
                <div class="columns">
                    <div class="column is-right">
                    {"My Blog"}
                    </div>
                    <div class="column">
                    {""}
                    </div>
                    <div class="column">
                    {""}
                    </div>
                    <div class="column">
                        <button class="button" onclick={ctx.link().callback(|_| Msg::Compose)}>
                            <span class="icon">
                                <i class="fab fa-github"></i>
                            </span>
                            <span>{"写博客/Compose"}</span>
                        </button>
                    </div>
                </div>
                <h1 class="title is-1">{ "博客/Posts" }</h1>
                <h2 class="subtitle">{ "All of your quality writing in one place" }</h2>
                <div class="columns">
                    <div class="column">
                        <ul class="list">
                            { self.view_posts(left_column_data) }
                        </ul>
                    </div>
                    <div class="column">
                        <ul class="list">
                            { self.view_posts(right_column_data) }
                        </ul>
                    </div>
                </div>
                <div class="container">
                    <nav class="pagination is-right" role="navigation" aria-label="pagination">
                        <a class="pagination-previous">
                            {"上一页/Previous"}
                        </a>
                        <a class="pagination-next">
                            {"下一页/Next page"}
                        </a>
                    </nav>
                </div>
            </>
        }
    }
}

impl PostList {
    fn view_posts(&self, posts: Vec<&PostDetail>) -> Html {
        posts.iter().map(|&post| html! {
            <li class="list-item mb-5">
                <div class="card">
                    <div class="card-image">
                        <figure class="image is-2by1">
                            <img alt="This post's image" src={post.title_image.clone()} loading="lazy" />
                        </figure>
                    </div>
                    <div class="card-content">
                        <Link<Route> classes={classes!("title", "is-block")} to={Route::ShowPost { id: post.id as u64 }}>
                            { &post.title }
                        </Link<Route>>
                    </div>
                </div>
            </li>
        }).collect()
    }
}