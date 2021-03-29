use alloc::{boxed::Box, string::String, vec::Vec};

use wasm_bindgen::prelude::*;
use yew::{
    agent::Bridged, html, services::fetch::FetchTask, Bridge, Callback, Component, ComponentLink, FocusEvent, Html,
    InputData, ShouldRender,
};
use yew_router::{agent::RouteRequest::ChangeRoute, prelude::*};

use blog_common::dto::post::{NewPost, PostDetail, Tag};

use crate::{
    app::AppRoute,
    component::error::ShowErrors,
    util::{request, Error},
    val,
};

#[wasm_bindgen(module = "/asset/editor.js")]
extern "C" {
    #[wasm_bindgen(js_name = initEditor)]
    fn init_editor();
    #[wasm_bindgen(js_name = getContent)]
    fn get_content();
}

pub(crate) struct Model {
    blog_params: NewPost,
    new_tags: String,
    error: Option<Error>,
    fetch_task: Option<FetchTask>,
    response: Callback<Result<PostDetail, Error>>,
    router_agent: Box<dyn Bridge<RouteAgent>>,
    all_tags: Vec<Tag>,
    link: ComponentLink<Self>,
}

pub(crate) enum Msg {
    AppendTag(String),
    Ignore,
    UpdateTitle(String),
    UpdateContent(String),
    UpdateNewTag(String),
    RemoveTag(String),
    Request,
    Response(Result<PostDetail, Error>),
    TagsResponse(Result<Vec<Tag>, Error>),
    InitEditor,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            blog_params: NewPost {
                title: String::default(),
                content: String::default(),
                tags: None,
            },
            new_tags: String::default(),
            error: None,
            fetch_task: None,
            // response: Default::default(),
            response: link.callback(Msg::Response),
            router_agent: RouteAgent::bridge(link.callback(|_| Msg::Ignore)),
            all_tags: Vec::new(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AppendTag(tag) => {
                self.new_tags.push_str(tag.as_str());
                self.new_tags.push(' ');
                return true;
            },
            Msg::Ignore => {},
            Msg::UpdateTitle(s) => self.blog_params.title = s,
            Msg::UpdateContent(s) => self.blog_params.content = s,
            Msg::UpdateNewTag(s) => self.new_tags = s,
            Msg::RemoveTag(s) => {
                if let Some(tags) = &mut self.blog_params.tags {
                    tags.retain(|t| t != &s);
                }
            },
            Msg::Request => {
                if !self.new_tags.is_empty() {
                    self.blog_params.tags = Some(
                        self.new_tags
                            .trim()
                            .split(|c| c == ' ')
                            .map(|s| String::from(s))
                            .collect(),
                    );
                }
                let fetch_task = request::post::<NewPost, PostDetail>(
                    val::BLOG_SAVE_URL,
                    self.blog_params.clone(),
                    self.response.clone(),
                );
                self.fetch_task = Some(fetch_task);
            },
            Msg::Response(Ok::<PostDetail, _>(blog)) => {
                self.fetch_task = None;
                self.router_agent.send(ChangeRoute(AppRoute::BlogShow(blog.id).into()));
            },
            Msg::Response(Err::<_, Error>(err)) => {
                self.error = Some(err);
                self.fetch_task = None;
                return true;
            },
            Msg::TagsResponse(Ok::<Vec<Tag>, _>(tags)) => {
                self.fetch_task = None;
                self.all_tags = tags;
                return true;
            },
            Msg::TagsResponse(Err::<_, Error>(err)) => {
                self.error = Some(err);
                self.fetch_task = None;
                return true;
            },
            Msg::InitEditor => {
                init_editor();
                return true;
            },
        }
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender { false }

    fn view(&self) -> Html {
        html! {
            <>
                <form class="pure-form pure-form-stacked" onsubmit=self.link.callback(|ev: FocusEvent| {
                    ev.prevent_default();
                    Msg::Request
                })>
                    <fieldset>
                        <input
                            class="pure-input-2-3"
                            type="text"
                            placeholder="Title"
                            value=&self.blog_params.title
                            oninput=self.link.callback(|e: InputData| Msg::UpdateTitle(e.value))
                            />
                        // <textarea
                        //     class="pure-input-2-3"
                        //     rows="16"
                        //     placeholder="Write your article (in markdown)"
                        //     value={&self.blog_params.content}
                        //     oninput=self.link.callback(|e: InputData| Msg::UpdateContent(e.value))>
                        // </textarea>
                        <div id="editor"></div>
                        <RouterAnchor<AppRoute> route=AppRoute::BlogUpload> {"Upload image"} </RouterAnchor<AppRoute>>
                        <input
                            class="pure-input-2-3"
                            type="text"
                            placeholder="Tags"
                            value=&self.new_tags
                            oninput=self.link.callback(|e: InputData| Msg::UpdateNewTag(e.value))
                            />
                        <div class="tag-list">
                            {
                                html! {for self.all_tags.iter().map(|tag| {
                                    let t = tag.name.clone();
                                    let append_tag = self.link.callback(move |ev| Msg::AppendTag(t.to_string()));
                                    html! {
                                        <span class="tag-btn pure-button" onclick=append_tag>
                                            { &tag.name }
                                        </span>
                                    }
                                })}
                            }
                        </div>
                        <br/>
                        <button
                            class="pure-button pure-button-primary"
                            type="submit"
                            disabled=false>
                            { "Publish" }
                        </button>
                    </fieldset>
                </form>
                <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.48.4/codemirror.min.css" />
                <link rel="stylesheet" href="https://uicdn.toast.com/editor/latest/toastui-editor.min.css" />
                <script src="https://uicdn.toast.com/editor/latest/toastui-editor-all.min.js" onload=self.link.callback(|_| Msg::InitEditor)></script>
            </>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let task = request::get::<Vec<Tag>>(val::BLOG_TAGS_URL, self.link.callback(Msg::TagsResponse));
            self.fetch_task = Some(task);
        }
    }
}
