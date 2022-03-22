use std::rc::Rc;

use blog_common::dto::post::PostDetail as PostDetailDto;
use blog_common::dto::Response;
use yew::prelude::*;
use yew_router::prelude::*;

pub struct Unauthorized {}

impl Component for Unauthorized {
    type Message = ();
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let loc = ctx.link().location().unwrap();
        let redirect_url = loc.path();
        let redirect_url = urlencoding::encode(loc.path());
        // let redirect_url = redirect_url.into_owned();
        let mut url = String::from("/management?.redirect_url=");
        url.push_str(redirect_url.as_ref());
        html! {
            <section class="hero is-danger is-bold is-medium">
                <div class="hero-body">
                    <div class="container">
                        <h1 class="title">
                            { "需要登录/Unauthorized" }
                        </h1>
                        <h2 class="subtitle">
                            <a href={url}>{ "请点击这里登录/Please click here to sign in." }</a>
                        </h2>
                    </div>
                </div>
            </section>
        }
    }
}
