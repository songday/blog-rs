use blog_common::dto::Response;
use yew::prelude::*;
use yew_router::prelude::*;
use weblog::*;

pub enum Msg {
    Compose,
}

pub struct PostList;

impl Component for PostList {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Compose => {
                wasm_bindgen_futures::spawn_local(async move {
                    let response = reqwasm::http::Request::get("/post/new").send().await.unwrap();
                    let json: Response<u64> = response.json().await.unwrap();
                    if json.status == 0 {
                        yew_router::push_route(crate::router::Route::ComposePost {id:json.data.unwrap()});
                    } else {
                        if let Some(loc) = web_sys::window().map(|window| window.location()) {
                            let _ = loc.set_href("/management");
                        } else {
                            console_log!("get location failed");
                        }
                    }
                });
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // let page = self.current_page();

        html! {
            <div class="section container">
            <div class="columns">
              <div class="column is-right">
                {"My Blog"}
              </div>
              <div class="column">
                {""}
              </div>
              <div class="column">
                {""}
              </div>
              <div class="column">
                <button class="button" onclick={ctx.link().callback(|_| Msg::Compose)}>
    <span class="icon">
      <i class="fab fa-github"></i>
    </span>
    <span>{"写博客"}</span>
  </button>
              </div>
            </div>
                <h1 class="title">{ "Posts" }</h1>
                <h2 class="subtitle">{ "All of our quality writing in one place" }</h2>
            </div>
        }
    }
}