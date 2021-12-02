use std::rc::Rc;

use blog_common::dto::post::PostDetail as PostDetailDto;
use blog_common::dto::Response;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct ShowDetailProps {
    pub post: Rc<PostDetailDto>,
}

#[function_component(ShowDetail)]
fn app(ShowDetailProps { post } : &ShowDetailProps) -> Html {
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
    post_id: u64,
}

impl Component for PostDetail {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { post_id: 0 }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.post_id != ctx.props().post_id
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let Self { post_id } = self;
        let detail_url = format!("/post/show/{}", post_id);
        let post = use_state(|| PostDetailDto::default());
        {
            let post = post.clone();
            use_effect_with_deps(
                move |_| {
                    let post = post.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let response: Response<PostDetailDto> = reqwasm::http::Request::get(&detail_url)
                            .send()
                            .await
                            .unwrap()
                            .json()
                            .await
                            .unwrap();
                        post.set(response.data.unwrap());
                    });
                    || ()
                },
                (),
            );
        }
        let post = (*post).clone();
        let post = Rc::new(post);

        html! {
            <>
                <ShowDetail post={post.clone()} />
            </>
        }
    }
}
