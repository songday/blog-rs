use blog_common::dto::post::PostDetail as PostDetailDto;
use blog_common::dto::Response;
use time::format_description;
use time::OffsetDateTime;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;

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
        use_effect(move || {
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
        });
    }
    let post = (*post_detail).clone();
    let title_image = post.title_image.to_string();
    let datetime = OffsetDateTime::from_unix_timestamp(post.created_at as i64).unwrap();
    let format = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
    let post_time = datetime.format(&format).expect("Failed to format the date");
    gloo::utils::document().set_title(&post.title);
    html! {
        <>
            <section class="hero is-large is-light has-background">
                <img src={ title_image } class="hero-background is-transparent"/>
                <div class="hero-body">
                    <div class="container">
                        <p class="title is-1">
                            { &post.title }
                        </p>
                        <p class="subtitle is-3">
                            { &post_time }
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
    type Message = ();
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

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let Self { post_id } = self;
        html! {
            <>
                <script type="application/javascript">
                {"
                document.addEventListener('DOMContentLoaded', () => {
                    (document.querySelectorAll('.notification .delete') || []).forEach(($delete) => {
                        const $notification = $delete.parentNode;                    
                        $delete.addEventListener('click', () => {
                        $notification.parentNode.removeChild($notification);
                        });
                    });
                });
                function delete(id) {
                    if (confirm('是否删除'))
                        location.href = '/post/delete/"}{post_id}{"';
                }
                "}
                </script>
                <ShowDetail post_id={*post_id} />
                <div class="container">
                    <div class="buttons are-small">
                        <Link<Route> classes={classes!("button")} to={Route::ComposePost { id: *post_id }}>
                            { "编辑/Edit" }
                        </Link<Route>>
                        <button class="button is-danger is-outlined">{"删除/Delete"}</button>
                    </div>
                </div>
                <div class="notification is-danger">
                    <button class="delete"></button>
                    { "删除后，数据将不能回复" }<br/>
                    { "Data cannot be recovered" }<br/>
                    <div class="buttons">
                        <button class="button is-danger is-outlined">{"删除/Delete"}</button>
                        <button class="button is-success">{"放弃/Cancel"}</button>
                    </div>
                </div>
            </>
        }
    }
}
