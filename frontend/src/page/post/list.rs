use yew::prelude::*;

use crate::component::PostsListComponent;

pub struct PostsList {}

impl Component for PostsList {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        gloo::utils::document().set_title("博客列表/Posts list");

        html! {
            <>
                <div class="columns">
                    <div class="column is-12">
                        <h1 class="title is-1">{ "博客列表/Posts list" }</h1>
                        <h2 class="subtitle">{ "All of your quality writing in one place" }</h2>
                    </div>
                </div>
                <PostsListComponent request_uri={"/post/list/".to_string()} />
            </>
        }
    }
}
