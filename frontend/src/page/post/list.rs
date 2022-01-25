use std::rc::Rc;
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
    max_id: i64,
    set_max_id_callback: Callback<i64>,
}

#[function_component(PostsListComponent)]
fn posts_list(
    PostsListComponentProps {
        max_id,
        set_max_id_callback,
    }: &PostsListComponentProps,
) -> Html {
    console_log!("pass in max_id=", max_id.to_string());
    let pagination_state = use_reducer(PaginationState::default);
    let posts: UseStateHandle<Vec<PostDetail>> = use_state(|| Vec::with_capacity(0));
    {
        let posts = posts.clone();
        let mut uri = String::with_capacity(32);
        uri.push_str("/post/list/");
        match pagination_state.pagination_type {
            PaginationType::PREV(id) => {
                uri.push_str("prev/");
                uri.push_str(id.to_string().as_str());
            },
            PaginationType::NEXT(id) => {
                uri.push_str("next/");
                uri.push_str(id.to_string().as_str());
            },
        }
        console_log!("uri=", &uri);
        use_effect_with_deps(
            move |_pagination_state| {
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
            },
            pagination_state.clone(),
        );
    }
    let posts = (*posts).clone();
    let len = posts.len();
    if len == 0 {
        return html! {};
    }
    let top_id = posts[0].id;
    let bottom_id = posts[len - 1].id;
    let max_id = pagination_state.max_post_id;
    if max_id == 0 || max_id < top_id {
        set_max_id_callback.emit(top_id);
    }
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
    let prev_disabled = if max_id < top_id { true } else { false };
    let next_disabled = if len < val::POSTS_PAGE_SIZE as usize {
        true
    } else {
        false
    };
    let prev = {
        let pagination_state = pagination_state.clone();
        Callback::from(move |_| pagination_state.dispatch(PaginationType::PREV(top_id)))
    };
    let next = {
        let pagination_state = pagination_state.clone();
        Callback::from(move |_| pagination_state.dispatch(PaginationType::NEXT(bottom_id)))
    };
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
            <div class="container">
                <nav class="pagination is-right" role="navigation" aria-label="pagination">
                    <a class="pagination-previous" disabled={prev_disabled} onclick={prev}>
                        {"上一页/Previous"}
                    </a>
                    <a class="pagination-next" disabled={next_disabled} onclick={next}>
                        {"下一页/Next page"}
                    </a>
                </nav>
            </div>
        </>
    }
}

#[derive(PartialEq)]
struct PaginationState {
    max_post_id: i64,
    pagination_type: PaginationType,
}

impl Default for PaginationState {
    fn default() -> Self {
        Self {
            max_post_id: 0,
            pagination_type: PaginationType::NEXT(0),
        }
    }
}

impl Reducible for PaginationState {
    type Action = PaginationType;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let new_max_post_id = match action {
            PaginationType::NEXT(top_id) => {
                if self.max_post_id < top_id {
                    top_id
                } else {
                    self.max_post_id
                }
            },
            _ => self.max_post_id,
        };

        Self {
            max_post_id: new_max_post_id,
            pagination_type: action,
        }
        .into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaginationType {
    PREV(i64),
    NEXT(i64),
}

pub enum Msg {
    Compose,
    SetMaxId(i64),
}

pub struct PostsList {
    max_id: i64,
}

impl Component for PostsList {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { max_id: 0 }
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
            Msg::SetMaxId(max_id) => {
                console_log!("SetMaxId=", max_id);
                self.max_id = max_id;
            },
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Self { max_id } = self;

        let set_max_id_callback = ctx.link().callback(move |new_max_id| Msg::SetMaxId(new_max_id));

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
                <PostsListComponent max_id={*max_id} set_max_id_callback={set_max_id_callback.clone()} />
            </>
        }
    }
}
