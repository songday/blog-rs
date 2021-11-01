use yew::prelude::*;
use yew_router::prelude::*;

use crate::page::{post::{PostCompose, PostDetail, PostList}};

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/posts/:id")]
    ShowPost { id: u64 },
    #[at("/posts/compose/:id")]
    ComposePost { id: u64 },
    #[at("/")]
    ListPosts,
    #[not_found]
    #[at("/#/404")]
    NotFound,
}

#[function_component(NotFound)]
fn not_found() -> Html {
    html! {
        <section class="hero is-danger is-bold is-large">
            <div class="hero-body">
                <div class="container">
                    <h1 class="title">
                        { "Page not found" }
                    </h1>
                    <h2 class="subtitle">
                        { "Page page does not seem to exist" }
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
        }
        Route::ListPosts => {
            html! { <PostList /> }
        }
        Route::ComposePost { id } => {
            html! { <PostCompose post_id={*id} /> }
        }
        _ => {
            html! { <NotFound /> }
        }
    }
}