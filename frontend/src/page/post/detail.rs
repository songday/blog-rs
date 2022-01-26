use blog_common::dto::post::PostDetail as PostDetailDto;
use blog_common::dto::Response;
use time::format_description;
use time::OffsetDateTime;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;

#[wasm_bindgen(module = "/asset/show.js")]
extern "C" {
    #[wasm_bindgen(js_name = showNotificationBox)]
    fn show_notification_box();
    #[wasm_bindgen(js_name = hideNotificationBox)]
    fn hide_notification_box(event: MouseEvent);
}

fn show_tags(post: &mut PostDetailDto) -> Html {
    if post.tags.is_none() {
        return html!{};
    }
    let tags = std::mem::replace(&mut post.tags, None);
    let tags = tags.unwrap().iter().map(|t| html!{<span class="tag is-info">{t}</span>}).collect::<Html>();
    html! {
        <div class="tags">
            {tags}
        </div>
    }
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
        use_effect_with_deps(move |_| {
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
        }, ());
    }
    let mut post = (*post_detail).clone();
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
                        {show_tags(&mut post)}
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
        weblog::console_log!("show_detail");
        let Self { post_id } = self;
        let mut delete_post_uri = String::with_capacity(32);
        delete_post_uri.push_str("/post/delete/");
        delete_post_uri.push_str(post_id.to_string().as_str());

        let show_notification_callback = Callback::from(|_: MouseEvent| show_notification_box());
        let hide_notification_callback = Callback::from(|e: MouseEvent| hide_notification_box(e));
        html! {
            <>
                <ShowDetail post_id={*post_id} />
                <div class="container">
                    <div class="buttons are-small">
                        <Link<Route> classes={classes!("button")} to={Route::ComposePost { id: *post_id }}>
                            { "编辑/Edit" }
                        </Link<Route>>
                        <button class="button is-danger is-outlined" onclick={show_notification_callback}>{"删除/Delete"}</button>
                    </div>
                    <div id="notification" class="notification is-danger is-light" style="display:none;width:435px">
                        <button class="delete" onclick={hide_notification_callback.clone()}></button>
                        { "删除后，数据将不能恢复/Data cannot be recovered" }<br/>
                        <div class="buttons">
                            <a class="button is-danger is-outlined" href={delete_post_uri}>{"删除/Delete"}</a>
                            <button class="button is-success" onclick={hide_notification_callback}>{"放弃/Cancel"}</button>
                        </div>
                    </div>
                </div>
            </>
        }
    }
}
