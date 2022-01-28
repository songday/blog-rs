use std::vec::Vec;

use blog_common::dto::post::PostDetail;
use blog_common::dto::{PaginationData, Response};
use blog_common::val;
use weblog::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;

#[function_component(TagsListComponent)]
fn tags_list() -> Html {
    let tags: UseStateHandle<Vec<String>> = use_state(|| Vec::with_capacity(0));
    {
        let tags = tags.clone();
        let mut uri = String::with_capacity(32);
        uri.push_str("/tags/all");
        use_effect_with_deps(
            move |_| {
                let tags = tags.clone();
                console_log!("request uri");
                wasm_bindgen_futures::spawn_local(async move {
                    let response: Response<Vec<String>> = reqwasm::http::Request::get(uri.as_str())
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    tags.set(response.data.unwrap());
                });
                || ()
            },
            (),
        );
    }
    let tags = (*tags).clone();
    let len = tags.len();
    if len == 0 {
        return html! {};
    }
    let mut classes = String::with_capacity(32);
    let tags = tags
        .iter()
        .map(|t| {
            let mut roll: usize = fastrand::usize(..val::TAG_SIZES.len());
            classes.push_str("tag is-light");
            classes.push_str(val::TAG_SIZES[roll]);
            roll = fastrand::usize(..val::TAG_COLORS.len());
            classes.push_str(val::TAG_COLORS[roll]);
            let html = html! {
                <span class={classes!(&classes)}>
                    <Link<Route> to={Route::ListPostsByTag { tag_name: String::from(t) }}>
                        { t }
                    </Link<Route>>
                </span>
            };
            classes.clear();
            html
        })
        .collect::<Html>();
    html! {
        <>
            <div class="container">
                <div class="tags are-large">
                    {tags}
                </div>
            </div>
        </>
    }
}

pub struct TagsList;

impl Component for TagsList {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        gloo::utils::document().set_title("所有标签/All tags");

        html! {
            <>
                <div class="columns">
                    <div class="column is-10">
                        <h1 class="title is-1">{ "所有标签/All tags" }</h1>
                        <h2 class="subtitle">{ "&nbsp;" }</h2>
                    </div>
                </div>
                <TagsListComponent />
            </>
        }
    }
}
