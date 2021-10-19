use yew::prelude::*;
use yew_router::prelude::*;

use crate::page::{Home, post::{PostCompose, PostDetail, PostList}};

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/#/posts/:id")]
    Post { id: u64 },
    #[at("/#/posts")]
    Posts,
    #[at("/posts/compose")]
    ComposePost,
    #[at("/#/posts/:id/edit")]
    EditPost { id: u64 },
    #[at("/#/easter-egg")]
    EasterEgg,
    #[at("/")]
    Home,
    #[not_found]
    #[at("/#/404")]
    NotFound,
}

#[function_component(EasterEgg)]
fn easter_egg() -> Html {
    html! {
        <h1>{ "Congrats, You've found an Easter Egg!" }</h1>
    }
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
        Route::Post { id } => {
            html! { <PostDetail post_id={*id} /> }
        }
        Route::Posts => {
            html! { <PostList /> }
        }
        Route::ComposePost => {
            html! { <PostCompose post_id={None} /> }
        }
        Route::EditPost { id } => {
            html! { <PostCompose post_id={*id} /> }
        }
        Route::Home => {
            html! { <Home /> }
        }
        Route::EasterEgg => {
            html! { <EasterEgg /> }
        }
        Route::NotFound => {
            html! { <NotFound /> }
        }
    }
}