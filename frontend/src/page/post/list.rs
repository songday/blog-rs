use std::vec::Vec;

use blog_common::dto::post::PostDetail;
use blog_common::dto::{PaginationData, Response};
use blog_common::val;
use weblog::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;

fn view_posts(posts: Vec<&PostDetail>) -> Html {
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

#[derive(PartialEq, Properties)]
pub struct PostsListComponentProps {
    max_id: u64,
    top_id: u64,
    bottom_id: u64,
    pagination_type: PaginationType,
    set_max_id_callback: Callback<u64>,
    set_pagination_id_callback: Callback<(u64, u64)>,
}

#[function_component(PostsListComponent)]
fn posts_list(
    PostsListComponentProps {
        max_id,
        top_id,
        bottom_id,
        pagination_type,
        set_max_id_callback,
        set_pagination_id_callback,
    }: &PostsListComponentProps,
) -> Html {
    console_log!("pagination_type=", format!("{:?}",pagination_type), ",top_id=",top_id.to_string(), ",bottom_id=",bottom_id.to_string());
    let posts = use_state(|| vec![]);
    {
        let posts = posts.clone();
        let mut uri = String::with_capacity(16);
        uri.push_str("/post/list/");
        match pagination_type {
            PaginationType::PREV => {
                uri.push_str("prev/");
                uri.push_str(top_id.to_string().as_str());
            }
            PaginationType::NEXT => {
                uri.push_str("next/");
                uri.push_str(bottom_id.to_string().as_str());
            }
        }
        console_log!("uri=", &uri);
        use_effect(
            move || {
                let posts = posts.clone();
                console_log!("request uri");
                wasm_bindgen_futures::spawn_local(async move {
                    let response: Response<PaginationData<Vec<PostDetail>>> = reqwasm::http::Request::get(uri.as_str())
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    posts.set(response.data.unwrap().data);
                });
                || ()
            }
        );
    }
    let posts = (*posts).clone();
    let len = posts.len();
    if len == 0 {
        return html! {};
    }
    let max_id = *max_id;
    let top_id = posts[0].id as u64;
    if max_id == 0 || max_id < top_id {
        set_max_id_callback.emit(top_id);
    }
    set_pagination_id_callback.emit((top_id, posts[len - 1].id as u64));
    let row_num = len / 2 + 1;
    let mut left_column_data: Vec<&PostDetail> = Vec::with_capacity(row_num);
    let mut right_column_data: Vec<&PostDetail> = Vec::with_capacity(row_num);
    let mut is_odd = true;
    for post in posts.iter() {
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
                <div class="column">
                    <ul class="list">
                        { view_posts(left_column_data) }
                    </ul>
                </div>
                <div class="column">
                    <ul class="list">
                        { view_posts(right_column_data) }
                    </ul>
                </div>
            </div>
        </>
    }
}


#[derive(PartialEq, Properties)]
pub struct PaginationComponentProps {
    max_id: u64,
    top_id: u64,
    prev: Callback<MouseEvent>,
    next: Callback<MouseEvent>,
}

#[function_component(PaginationButtons)]
fn pagination_buttons(PaginationComponentProps{max_id, top_id, prev, next}: &PaginationComponentProps) -> Html {
    let prev_disabled = if max_id > top_id {""} else {"disabled"};
    html!{
        <div class="container">
            <nav class="pagination is-right" role="navigation" aria-label="pagination">
                <a class="pagination-previous" {prev_disabled} onclick={prev}>
                    {"上一页/Previous"}
                </a>
                <a class="pagination-next" onclick={next}>
                    {"下一页/Next page"}
                </a>
            </nav>
        </div>
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PaginationType {
    PREV,
    NEXT,
}

pub enum Msg {
    Compose,
    SetMaxId(u64),
    SetPaginationId(u64, u64),
    Pagination(PaginationType),
}

pub struct PostsList {
    max_id: u64,
    top_id: u64,
    bottom_id: u64,
    pagination_type: PaginationType,
}

impl Component for PostsList {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            max_id: 0,
            top_id: 0,
            bottom_id: 0,
            pagination_type: PaginationType::NEXT,
        }
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
            }
            Msg::Pagination(pagination_type) => {
                self.pagination_type = pagination_type;
                return true;
            }
            Msg::SetMaxId(max_id) => {
                self.max_id = max_id;
            }
            Msg::SetPaginationId(top_id, bottom_id) => {
                self.top_id = top_id;
                self.bottom_id = bottom_id;
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Self {
            max_id,
            top_id,
            bottom_id,
            pagination_type,
        } = self;

        let set_max_id_callback = ctx.link().callback(move |max_id| Msg::SetMaxId(max_id));
        let set_pagination_id_callback = ctx
            .link()
            .callback(move |(top_id, bottom_id)| Msg::SetPaginationId(top_id, bottom_id));
        let prev = ctx.link().callback(|_| Msg::Pagination(PaginationType::PREV));
        let next = ctx.link().callback(|_| Msg::Pagination(PaginationType::NEXT));

        gloo::utils::document().set_title("博客/Blog");

        html! {
            <>
                <div class="columns">
                    <div class="column is-10">
                        <h1 class="title is-1">{ "博客/Posts" }</h1>
                        <h2 class="subtitle">{ "All of your quality writing in one place" }</h2>
                    </div>
                    <div class="column" style="text-align:right">
                        <button class="button" onclick={ctx.link().callback(|_| Msg::Compose)}>
                            <span class="icon">
                                <i class="far fa-edit"></i>
                            </span>
                            <span>{"写博客/Compose"}</span>
                        </button>
                    </div>
                </div>
                <PostsListComponent max_id={*max_id} top_id={*top_id} bottom_id={*bottom_id} pagination_type={pagination_type.clone()} set_max_id_callback={set_max_id_callback.clone()} set_pagination_id_callback={set_pagination_id_callback.clone()} />
                <div class="container">
                    <nav class="pagination is-right" role="navigation" aria-label="pagination">
                        <PaginationButtons max_id={*max_id} top_id={*top_id} prev={prev} next={next}/>
                    </nav>
                </div>
            </>
        }
    }
}
