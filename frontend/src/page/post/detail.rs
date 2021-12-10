use std::rc::Rc;

use blog_common::dto::post::PostDetail as PostDetailDto;
use blog_common::dto::Response;
use yew::prelude::*;
use yew_router::prelude::*;

pub enum Msg {
    DisplayDetail(u64),
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct ShowDetailProps {
    pub post_id: u64,
}

#[function_component(ShowDetail)]
fn app(ShowDetailProps { post_id }: &ShowDetailProps) -> Html {
    let detail_url = format!("/post/show/{}", post_id);
    let post_detail = use_state(|| PostDetailDto::default());
    {
        let post_detail = post_detail.clone();
        use_effect_with_deps(
            move |_| {
                let post_detail = post_detail.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let response: Response<PostDetailDto> = reqwasm::http::Request::get(&detail_url)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    post_detail.set(response.data.unwrap());
                });
                || ()
            },
            (),
        );
    }
    let post = (*post_detail).clone();
    let title_image = post.title_image.to_string();
    html! {
        <>
            <section class="hero is-medium is-light has-background">
                <img src={ title_image } class="hero-background is-transparent"/>
                <div class="hero-body">
                    <div class="container">
                        <p class="title">
                            { &post.title }
                        </p>
                        <p class="subtitle">
                            {"Medium subtitle"}
                        </p>
                        <div class="tags">
                            <span class="tag is-info">{"tag"}</span>
                        </div>
                    </div>
                </div>
            </section>
            <div class="section container">
                <article class="media block box my-6">
                    <div class="media-content">
                        <div class="content">
                            <p class="is-family-secondary">
                                { &post.content }
                            </p>
                        </div>
                    </div>
                </article>
            </div>
        </>
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    pub post_id: u64,
}

pub struct PostDetail {
    pub post_id: u64,
}

impl Component for PostDetail {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            post_id: ctx.props().post_id,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let changed = self.post_id != ctx.props().post_id;
        if changed {
            weblog::console_log!("changed to load");
            self.post_id = ctx.props().post_id;
        }
        changed
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DisplayDetail(post_id) => {
                weblog::console_log!("load post ", post_id);
                return true;
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let Self { post_id } = self;
        html! {
            <>
                <ShowDetail post_id={*post_id} />
            </>
        }
    }
}
