use yew::prelude::*;
use yew_router::prelude::*;

use crate::page::post::{PostCompose, PostDetail, PostsList, PostsListByTag};
use crate::page::tag::TagsList;

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/posts/:id")]
    ShowPost { id: u64 },
    #[at("/posts/compose/:id")]
    ComposePost { id: u64 },
    #[at("/posts/tag/:tag_name")]
    ListPostsByTag { tag_name: String },
    #[at("/tags")]
    Tags,
    #[at("/")]
    ListPosts,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(NotFound)]
fn not_found() -> Html {
    html! {
        <section class="hero is-danger is-bold is-large">
            <div class="hero-body">
                <div class="container">
                    <h1 class="title">
                        { "找不到请求的页面/Page not found" }
                    </h1>
                    <h2 class="subtitle">
                        { "找不到请求的页面/Page page does not seem to exist." }
                    </h2>
                </div>
            </div>
        </section>
    }
}

pub fn switch(routes: &Route) -> Html {
    match routes {
        Route::ShowPost { id } => {
            html! { <PostDetail post_id={*id} /> }
        },
        Route::ListPostsByTag { tag_name } => {
            html! { <PostsListByTag tag_name={String::from(tag_name)} /> }
        },
        Route::ListPosts => {
            html! { <PostsList /> }
        },
        Route::ComposePost { id } => {
            html! { <PostCompose post_id={*id} /> }
        },
        Route::Tags => {
            html! { <TagsList /> }
        },
        _ => {
            html! { <NotFound /> }
        },
    }
}
