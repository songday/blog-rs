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

#[derive(PartialEq, Properties)]
pub struct PostsListComponentProps {
    request_uri: String,
}

#[function_component(PostsListComponent)]
pub fn posts_list(
    PostsListComponentProps {
        request_uri,
    }: &PostsListComponentProps,
) -> Html {
    console_log!("pass in max_id=", max_id.to_string());
    let pagination_state = use_reducer(PaginationState::default);
    let posts: UseStateHandle<Vec<PostDetail>> = use_state(|| Vec::with_capacity(0));
    {
        let posts = posts.clone();
        let mut uri = String::with_capacity(32);
        uri.push_str(request_uri);
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
