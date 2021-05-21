use alloc::{boxed::Box, string::String, vec::Vec};

use wasm_bindgen::prelude::*;
use yew::{
    agent::Bridged,
    html,
    services::{
        fetch::FetchTask,
        reader::{File, FileChunk, FileData, ReaderService, ReaderTask},
    },
    Bridge, Callback, ChangeData, Component, ComponentLink, FocusEvent, Html, InputData, ShouldRender,
};
use yew_router::{agent::RouteRequest::ChangeRoute, prelude::*};

use blog_common::dto::post::{NewPost, PostDetail, Tag};

use crate::{
    app::AppRoute,
    component::error::ShowErrors,
    util::{request, Error},
    val,
};
use yew::services::ConsoleService;

#[wasm_bindgen(module = "/asset/editor.js")]
extern "C" {
    #[wasm_bindgen(js_name = initEditor)]
    fn init_editor();
    #[wasm_bindgen(js_name = getContent)]
    fn get_content() -> String;
    #[wasm_bindgen(js_name = inputTag)]
    fn input_tag(event: web_sys::KeyboardEvent);
    #[wasm_bindgen(js_name = selectTag)]
    fn select_tag(tag: String);
    #[wasm_bindgen(js_name = getSelectedTags)]
    fn get_selected_tags() -> Vec<wasm_bindgen::JsValue>;
}

pub(crate) struct Model {
    blog_params: NewPost,
    error: Option<Error>,
    fetch_task: Option<FetchTask>,
    response: Callback<Result<PostDetail, Error>>,
    router_agent: Box<dyn Bridge<RouteAgent>>,
    all_tags: Vec<String>,
    link: ComponentLink<Self>,
}

pub(crate) enum Msg {
    SelectTag(String),
    Ignore,
    UpdateTitle(String),
    // UpdateContent(String),
    InputNewTag(web_sys::KeyboardEvent),
    Request,
    Response(Result<PostDetail, Error>),
    TagsResponse(Result<Vec<String>, Error>),
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
            Msg::SelectTag(tag) => {
                select_tag(tag);
                return false;
            },
            Msg::Ignore => {},
            Msg::UpdateTitle(s) => self.blog_params.title = s,
            Msg::InputNewTag(e) => input_tag(e),
            Msg::Request => {
                self.blog_params.content = get_content();
                ConsoleService::log(&self.blog_params.content);
                self.blog_params.tags = Some(get_selected_tags().iter().map(|e| e.as_string().unwrap()).collect());
                let fetch_task = request::post::<NewPost, PostDetail>(
                    val::BLOG_SAVE_URI,
                    self.blog_params.clone(),
                    self.response.clone(),
                );
                self.fetch_task = Some(fetch_task);
            },
            Msg::Response(Ok::<PostDetail, _>(blog)) => {
                self.fetch_task = None;
                self.router_agent.send(ChangeRoute(AppRoute::PostShow(blog.id).into()));
            },
            Msg::Response(Err::<_, Error>(err)) => {
                self.error = Some(err);
                self.fetch_task = None;
                return true;
            },
            Msg::TagsResponse(Ok::<Vec<String>, _>(tags)) => {
                ConsoleService::log(tags.len().to_string().as_str());
                self.fetch_task = None;
                self.all_tags = tags;
                return true;
            },
            Msg::TagsResponse(Err::<_, Error>(err)) => {
                ConsoleService::log("error");
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
                <div class="row">
                    <div class="col">
                        <h1>{"新增博客"}</h1>
                    </div>
                </div>
                <form class="row g-3" onsubmit=self.link.callback(|ev: FocusEvent| {
                    ev.prevent_default();
                    Msg::Ignore
                })>
                    <div class="col-12">
                        <label class="form-label">{"标题"}</label>
                        <input
                            class="form-control"
                            type="text"
                            value=&self.blog_params.title
                            oninput=self.link.callback(|e: InputData| Msg::UpdateTitle(e.value))
                            />
                    </div>
                    // <textarea
                    //     class="pure-input-2-3"
                    //     rows="16"
                    //     placeholder="Write your article (in markdown)"
                    //     value={&self.blog_params.content}
                    //     oninput=self.link.callback(|e: InputData| Msg::UpdateContent(e.value))>
                    // </textarea>
                    <div class="col-12">
                        <label class="form-label">{"内容"}</label>
                        <div id="editor"></div>
                    </div>
                    // <RouterAnchor<AppRoute> route=AppRoute::BlogUpload> {"Upload image"} </RouterAnchor<AppRoute>>
                    <div class="col-12">
                        <label class="form-label">{"标签"}</label>
                        <div class="col-12 d-grid gap-2 d-md-block" id="tagsBox">
                        </div>
                        <div class="col-md-6">
                            {"新增标签："}<input id="tagInput"
                                type="text"
                                placeholder="不超过20字"
                                onkeyup=self.link.callback(|e: web_sys::KeyboardEvent| Msg::InputNewTag(e))
                                />{"（按回车新增）"}
                        </div>
                        <div class="col-12">
                            {"选择已有标签："}
                            {
                                html! {for self.all_tags.iter().map(|tag| {
                                    let t = String::from(tag);
                                    let select_tag = self.link.callback(move |ev| Msg::SelectTag(t.to_string()));
                                    html! {
                                        <button type="button" class="btn btn-outline-dark btn-sm ms-1" onclick=select_tag>
                                            { tag }
                                        </button>
                                    }
                                })}
                            }
                        </div>
                    </div>
                    <div class="col-12">
                        <button
                            class="btn btn-primary"
                            type="button"
                            onclick=self.link.callback(|_| Msg::Request)
                            disabled=false>
                            { "发布" }
                        </button>
                    </div>
                </form>
                <link rel="stylesheet" href="/asset/codemirror.min.css" />
                <link rel="stylesheet" href="/asset/toastui-editor.min.css" />
                <script src="/asset/toastui-editor-all.min.js" onload=self.link.callback(|_| Msg::InitEditor)></script>
            </>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let task = request::get::<Vec<String>>(val::TAG_LIST_URI, self.link.callback(Msg::TagsResponse));
            self.fetch_task = Some(task);
        }
    }
}
