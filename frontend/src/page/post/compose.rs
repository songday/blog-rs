use blog_common::dto::post::PostData;
use web_sys::HtmlInputElement;
use wasm_bindgen::prelude::*;
use weblog::*;
use yew::events::InputEvent;
use yew::prelude::*;
use yew_router::prelude::*;

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

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    pub post_id: u64,
}

pub struct PostCompose {
    post_id: u64,
    post_data: PostData,
}

pub enum Msg {
    Ignore,
    UpdateTitle(String),
    InitEditor,
    PostRequest,
}

impl Component for PostCompose {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            post_id: 0,
            post_data: PostData::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            // Msg::SelectTag(tag) => {
            //     select_tag(tag);
            //     return false;
            // },
            Msg::Ignore => {},
            Msg::UpdateTitle(s) => self.post_data.title = s,
            Msg::PostRequest => {
                self.post_data.content = get_content();
                console_log!(&self.post_data.content);
                // self.blog_params.tags = Some(get_selected_tags().iter().map(|e| e.as_string().unwrap()).collect());
                // let fetch_task = request::post::<NewPost, PostDetail>(
                //     val::BLOG_SAVE_URI,
                //     self.blog_params.clone(),
                //     self.response.clone(),
                // );
                // self.fetch_task = Some(fetch_task);
            },
            // Msg::Response(Ok::<PostDetail, _>(blog)) => {
            //     self.fetch_task = None;
            //     self.router_agent.send(ChangeRoute(AppRoute::PostShow(blog.id).into()));
            // },
            // Msg::Response(Err::<_, Error>(err)) => {
            //     self.error = Some(err);
            //     self.fetch_task = None;
            //     return true;
            // },
            // Msg::TagsResponse(Ok::<Vec<String>, _>(tags)) => {
            //     ConsoleService::log(tags.len().to_string().as_str());
            //     self.fetch_task = None;
            //     self.all_tags = tags;
            //     return true;
            // },
            // Msg::TagsResponse(Err::<_, Error>(err)) => {
            //     ConsoleService::log("error");
            //     self.error = Some(err);
            //     self.fetch_task = None;
            //     return true;
            // },
            Msg::InitEditor => {
                init_editor();
                return false;
            },
        }
        false
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.post_id != ctx.props().post_id
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Self { post_id, post_data } = self;

        html! {
            <>
                <h1>{"新增博客"}</h1>
                <form class="row g-3" onsubmit={ctx.link().callback(|ev: FocusEvent| {
                    ev.prevent_default();
                    Msg::Ignore
                })}>
                    <div class="field">
                      <label class="label">{"题图"}</label>
                      <div class="file is-boxed">
                        <label class="file-label">
                          <input class="file-input" type="file" name="resume">
                          <span class="file-cta">
                            <span class="file-icon">
                              <i class="fas fa-upload"></i>
                            </span>
                            <span class="file-label">
                              请选择一张图片…
                            </span>
                          </span>
                        </label>
                      </div>
                    </div>
                    <div class="field">
                      <label class="label">{"标题"}</label>
                      <div class="control">
                        <input class="input" type="text" placeholder="博客标题" value={self.post_data.title.clone()}
                                                oninput={ctx.link().callback(|e: InputEvent| {let input = e.target_unchecked_into::<HtmlInputElement>();Msg::UpdateTitle(input.value())})}/>
                      </div>
                    </div>
                    <div class="field">
                        <label class="form-label">{"内容"}</label>
                        <div id="editor"></div>
                    </div>
                    // <RouterAnchor<AppRoute> route=AppRoute::BlogUpload> {"Upload image"} </RouterAnchor<AppRoute>>
                    <div class="field">
                      <label class="label">{"标签"}</label>
                      <div class="control">
                        <input class="input" type="text" placeholder="回车添加"/>
                      </div>
                    </div>
                    <div class="field is-grouped">
                      <div class="control">
                        <button class="button is-link" onclick={ctx.link().callback(|_| Msg::PostRequest)}>{ "发布" }</button>
                      </div>
                      <div class="control">
                        <button class="button is-link is-light">{ "返回" }</button>
                      </div>
                    </div>
                </form>
                <link rel="stylesheet" href="/asset/codemirror.min.css" />
                <link rel="stylesheet" href="/asset/toastui-editor.min.css" />
                <script src="/asset/toastui-editor-all.min.js" onload={ctx.link().callback(|_| Msg::InitEditor)}></script>
            </>
        }
    }
}