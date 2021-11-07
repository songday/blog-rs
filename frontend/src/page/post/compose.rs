use std::collections::HashMap;

use blog_common::dto::post::PostData;
use gloo_file::callbacks::FileReader;
use gloo_file::File;
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
    #[wasm_bindgen(js_name = randomTitleImage)]
    fn random_title_image(id: u64);
    #[wasm_bindgen(js_name = goBack)]
    fn go_back();
    #[wasm_bindgen(js_name = uploadTitleImage)]
    fn upload_title_image(post_id: u64, files: Vec<web_sys::File>);
}

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    pub post_id: u64,
}

pub struct PostCompose {
    post_id: u64,
    post_data: PostData,
    readers: HashMap<String, FileReader>,
}

pub enum Msg {
    Ignore,
    UpdateTitle(String),
    InitEditor,
    PostRequest,
    LoadedBytes(String, Vec<u8>),
    Files(u64, Vec<web_sys::File>),
    RetrieveRandomTitleImage,
    GoBack,
}

impl Component for PostCompose {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            post_id: 0,
            post_data: PostData::default(),
            readers: HashMap::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            // Msg::SelectTag(tag) => {
            //     select_tag(tag);
            //     return false;
            // },
            Msg::Ignore => {}
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
            }
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
            Msg::LoadedBytes(file_name, data) => {
                let info = format!("file_name: {}, data: {:?}", file_name, data);
                console_log!(&info);
                // wasm_bindgen_futures::spawn_local(async move {
                //     let response = reqwasm::http::Request::post("").body(&data.as_slice()).send().await.unwrap();
                // });
                self.readers.remove(&file_name);
            }
            Msg::Files(post_id, files) => {
                // for file in files.into_iter() {
                    // let file_name = file.name();
                    // let task = {
                    //     let file_name = file_name.clone();
                    //     let link = ctx.link().clone();
                    //     gloo_file::callbacks::read_as_bytes(&file, move |res| {
                    //         link.send_message(Msg::LoadedBytes(
                    //             file_name,
                    //             res.expect("failed to read file"),
                    //         ))
                    //     })
                    // };
                    // self.readers.insert(file_name, task);
                // }
                upload_title_image(post_id, files);
            }
            Msg::InitEditor => {
                init_editor();
                return false;
            }
            Msg::RetrieveRandomTitleImage => {
                random_title_image(self.post_id);
            }
            Msg::GoBack => {
                go_back();
            }
        }
        false
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.post_id != ctx.props().post_id
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Self { post_id, post_data, readers } = self;
        let post_id = *post_id;

        html! {
            <>
            <div class="container">
                <h1 class="title is-1">{"新增博客"}</h1>
            </div>
            <p>{" "}</p>
                <form class="row g-3" onsubmit={ctx.link().callback(|ev: FocusEvent| {
                    ev.prevent_default();
                    Msg::Ignore
                })}>
            <div class="container">
                    <div class="field">
                      <label class="label">{"题图"}</label>
                    </div>
            <nav class="level">
              <p class="level-item has-text-centered">
    {""}
  </p>
<p class="level-item has-text-centered">
            <div class="file is-normal">
  <label class="file-label">
    <input class="file-input" multiple=false accept="image/*" type="file" name="title-image" onchange={ctx.link().callback(move |e: Event| {
                            let mut result = Vec::new();
                            let input: HtmlInputElement = e.target_unchecked_into();

                            if let Some(files) = input.files() {
                                let files = js_sys::try_iter(&files)
                                    .unwrap()
                                    .unwrap()
                                    .map(|v| web_sys::File::from(v.unwrap()))
                                    // .map(File::from)
                                    ;
                                result.extend(files);
                            }
                            Msg::Files(post_id, result)
                        })}/>
    <span class="file-cta">
      <span class="file-icon">
        <i class="fas fa-upload"></i>
      </span>
      <span class="file-label">
        {"上传图片"}
      </span>
    </span>
  </label>
</div>
  </p>
  <p class="level-item has-text-centered">
    {"或"}
  </p>
  <p class="level-item has-text-centered">
              <button class="button" onclick={ctx.link().callback(|_| Msg::RetrieveRandomTitleImage)}>
    <span class="icon">
      <i class="fas fa-download"></i>
    </span>
    <span>{"随机下载一张"}</span>
  </button>
  </p>
            <p class="level-item has-text-centered">
    {""}
  </p>
</nav>
                    </div>
                <section class="hero is-medium is-light has-background">
                  <img id="title-image" src="" class="hero-background is-transparent"/>
                </section>
            <div class="container">
                    <div class="field">
                      <label class="label">{"标题"}</label>
                      <div class="control">
                        <input class="input" type="text" placeholder="博客标题" value={self.post_data.title.clone()}
                                                oninput={ctx.link().callback(|e: InputEvent| {let input = e.target_unchecked_into::<HtmlInputElement>();Msg::UpdateTitle(input.value())})}/>
                      </div>
                    </div>
                    <div class="field">
                        <label class="label">{"内容"}</label>
                        <div id="editor"></div>
                    </div>
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
                        <button class="button is-link is-light" onclick={ctx.link().callback(|_| Msg::GoBack)}>{ "返回" }</button>
                      </div>
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