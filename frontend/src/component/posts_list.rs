use std::rc::Rc;
use std::vec::Vec;

use blog_common::dto::post::PostDetail;
use blog_common::dto::{PaginationData, Response};
use blog_common::val;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use weblog::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::i18n;
use crate::router::Route;

const POSTS_PAGE_SIZE: usize = val::POSTS_PAGE_SIZE as usize;

#[wasm_bindgen(module = "/asset/common.js")]
extern "C" {
    #[wasm_bindgen(js_name = userLanguage)]
    fn user_language() -> String;
}

fn view_posts(posts: Vec<&PostDetail>) -> Html {
    posts.iter().map(|&post| html! {
        <li class="list-item mb-5">
            <div class="card">
                <div class="card-image">
                    <figure class="image is-2by1">
                        <Link<Route> to={Route::ShowPost { id: post.id as u64 }}>
                            <img alt={ post.title.clone() } src={post.title_image.clone()} loading="lazy" />
                        </Link<Route>>
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

#[derive(Deserialize, Serialize)]
struct PaginationQuery {
    // pagination_type
    pub t: String,
    // page num
    pub pn: String,
    // top id
    pub tid: String,
    // bottom id
    pub bid: String,
}

impl From<PaginationQuery> for PaginationState {
    fn from(q: PaginationQuery) -> Self {
        let page_num = q.pn.parse::<i64>().unwrap_or(0);
        let pagination_type = match q.t.as_str() {
            "p" => PaginationType::PREV(q.tid.parse::<i64>().unwrap_or(0)),
            _ => PaginationType::NEXT(q.bid.parse::<i64>().unwrap_or(0)),
        };
        Self {
            page_num,
            pagination_type,
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct PostsListComponentProps {
    pub request_uri: String,
}

#[function_component(PostsListComponent)]
pub fn posts_list(PostsListComponentProps { request_uri }: &PostsListComponentProps) -> Html {
    let loc = use_location().unwrap();
    let nav = use_navigator().unwrap();
    let pagination_state = match loc.query::<PaginationQuery>() {
        Ok(q) => q.into(),
        Err(e) => PaginationState::default(),
    };
    // let m = format!("{:?}", &pagination_state);
    // console_log!(m);

    let pagination_state = use_reducer(|| pagination_state);
    let posts: UseStateHandle<Vec<PostDetail>> = use_state(|| Vec::with_capacity(0));
    {
        let posts = posts.clone();
        let mut uri = String::with_capacity(32);
        uri.push_str(request_uri);
        use_effect_with_deps(
            move |pagination_state| {
                match pagination_state.pagination_type {
                    PaginationType::PREV(id) => {
                        uri.push_str("prev/");
                        uri.push_str(id.to_string().as_str());
                    },
                    PaginationType::NEXT(bottom_id) => {
                        uri.push_str("next/");
                        uri.push_str(bottom_id.to_string().as_str());
                    },
                }
                console_log!("request uri=", &uri);

                let posts = posts.clone();
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
    let prev_disabled = pagination_state.page_num < 2;
    let next_disabled = len < POSTS_PAGE_SIZE;
    let prev = if prev_disabled {
        Callback::noop()
    } else {
        let pagination_state = pagination_state.clone();
        let nav = nav.clone();
        Callback::from(move |_| {
            nav.push_with_query(
                &Route::ListPosts,
                &PaginationQuery {
                    t: "p".to_string(),
                    pn: (pagination_state.page_num - 1).to_string(),
                    tid: top_id.to_string(),
                    bid: bottom_id.to_string(),
                },
            );
            pagination_state.dispatch(PaginationType::PREV(top_id))
        })
    };
    let next = if next_disabled {
        Callback::noop()
    } else {
        let pagination_state = pagination_state.clone();
        Callback::from(move |_| {
            nav.push_with_query(
                &Route::ListPosts,
                &PaginationQuery {
                    t: "n".to_string(),
                    pn: (pagination_state.page_num + 1).to_string(),
                    tid: top_id.to_string(),
                    bid: bottom_id.to_string(),
                },
            );
            pagination_state.dispatch(PaginationType::NEXT(bottom_id))
        })
    };
    web_sys::window().unwrap().scroll_to_with_x_and_y(0.0, 0.0);
    let messages = i18n::get(&user_language(), vec!["pp", "np"]).unwrap();
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
                        {messages.get("pp").unwrap()}
                    </a>
                    <a class="pagination-next" disabled={next_disabled} onclick={next}>
                        {messages.get("np").unwrap()}
                    </a>
                </nav>
            </div>
        </>
    }
}

#[derive(Debug)]
struct PaginationState {
    page_num: i64,
    pagination_type: PaginationType,
}

impl PartialEq for PaginationState {
    fn eq(&self, other: &Self) -> bool {
        // self.max_post_id == other.max_post_id && self.pagination_type.eq(&other.pagination_type)
        let r = self.page_num == other.page_num && self.pagination_type.eq(&other.pagination_type);
        console_log!("PaginationState_PartialEq", r);
        r
    }
}

impl Default for PaginationState {
    fn default() -> Self {
        Self {
            page_num: 1,
            pagination_type: PaginationType::NEXT(0),
        }
    }
}

impl Reducible for PaginationState {
    type Action = PaginationType;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let page_num = match action {
            PaginationType::NEXT(_bottom_id) => self.page_num + 1,
            _ => self.page_num - 1,
        };

        Self {
            page_num,
            pagination_type: action,
        }
        .into()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PaginationType {
    PREV(i64),
    NEXT(i64),
}

impl PartialEq for PaginationType {
    fn eq(&self, other: &Self) -> bool {
        use PaginationType::*;
        match (self, other) {
            (&PREV(ref a), &PREV(ref b)) => a == b,
            (&NEXT(ref a), &NEXT(ref b)) => a == b,
            _ => false,
        }
    }
}
