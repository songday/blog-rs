use alloc::{boxed::Box, string::String, vec::Vec};

use blog_common::dto::{
    post::{PostData, PostDetail, Tag},
    user::UserInfo,
};
use wasm_bindgen::prelude::*;
use yew::{
    agent::Bridged,
    html,
    services::{
        fetch::FetchTask,
        reader::{File, FileChunk, FileData, ReaderService, ReaderTask},
    },
    Bridge, Callback, ChangeData, Component, ComponentLink, FocusEvent, Html, InputData, Properties, ShouldRender,
};
use yew_router::{agent::RouteRequest::ChangeRoute, prelude::*};

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
    #[wasm_bindgen(js_name = setInitContent)]
    fn set_init_content(intent_content: String);
    #[wasm_bindgen(js_name = getContent)]
    fn get_content() -> String;
    #[wasm_bindgen(js_name = inputTag)]
    fn input_tag(event: web_sys::KeyboardEvent);
    #[wasm_bindgen(js_name = selectTag)]
    fn select_tag(tag: String);
    #[wasm_bindgen(js_name = selectTags)]
    fn select_tags(tag: Vec<wasm_bindgen::JsValue>);
    #[wasm_bindgen(js_name = getSelectedTags)]
    fn get_selected_tags() -> Vec<wasm_bindgen::JsValue>;
    #[wasm_bindgen(js_name = gotoLogin)]
    fn goto_login();
}

#[derive(Properties, Clone)]
pub struct Props {
    pub blog_id: Option<i64>,
    pub user_info: Option<UserInfo>,
}

pub(crate) struct Model {
    blog_id: Option<i64>,
    user_info: Option<UserInfo>,
    blog_params: PostData,
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
    EditPostResponse(Result<PostDetail, Error>),
    TagsResponse(Result<Vec<String>, Error>),
    InitEditor,
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            blog_id: props.blog_id,
            user_info: props.user_info,
            blog_params: PostData {
                id: None,
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
                // ConsoleService::log(&self.blog_params.content);
                self.blog_params.tags = Some(get_selected_tags().iter().map(|e| e.as_string().unwrap()).collect());
                let fetch_task = request::post::<PostData, PostDetail>(
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
                eprintln!("{:?}", err);
                self.error = Some(err);
                self.fetch_task = None;
                return true;
            },
            Msg::TagsResponse(Ok::<Vec<String>, _>(tags)) => {
                // ConsoleService::log(tags.len().to_string().as_str());
                self.all_tags = tags;
                if self.blog_id.is_some() {
                    let mut url = String::with_capacity(64);
                    url.push_str(val::BLOG_SHOW_URI);
                    url.push_str(self.blog_id.unwrap().to_string().as_str());
                    url.push_str("?edit=true");
                    let task = request::get::<PostDetail>(url.as_str(), self.link.callback(Msg::EditPostResponse));
                    self.fetch_task = Some(task);
                } else {
                    self.fetch_task = None;
                    return true;
                }
            },
            Msg::TagsResponse(Err::<_, Error>(err)) => {
                ConsoleService::log("error");
                eprintln!("{:?}", err);
                self.error = Some(err);
                self.fetch_task = None;
                return true;
            },
            Msg::EditPostResponse(Ok::<PostDetail, _>(mut post_detail)) => {
                self.blog_params.id = Some(post_detail.id);
                std::mem::swap(&mut self.blog_params.title, &mut post_detail.title);
                self.fetch_task = None;
                if post_detail.content.len() > 0 {
                    set_init_content(post_detail.content.clone());
                }
                if post_detail.tags.is_some() {
                    let val = post_detail
                        .tags
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|s| wasm_bindgen::JsValue::from_str(s))
                        .collect::<Vec<_>>();
                    select_tags(val);
                }
                return true;
            },
            Msg::EditPostResponse(Err::<_, Error>(err)) => {
                ConsoleService::log("error");
                eprintln!("{:?}", err);
                self.error = Some(err);
                self.fetch_task = None;
                return true;
            },
            Msg::InitEditor => {
                init_editor();
                let task = request::get::<Vec<String>>(val::TAG_LIST_URI, self.link.callback(Msg::TagsResponse));
                self.fetch_task = Some(task);
            },
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender { true }

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
                            value=self.blog_params.title.clone()
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
                                placeholder="不超过10字"
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
                // <link rel="stylesheet" href="/asset/codemirror.min.css" />
                // <link rel="stylesheet" href="/asset/toastui-editor.min.css" />
                <script src="/asset/toastui-editor-all.min.js" onload=self.link.callback(|_| Msg::InitEditor)></script>
            </>
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        if self.user_info.is_none() {
            goto_login();
        }
    }
}
