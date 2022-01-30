use weblog::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::{switch, Route};

pub enum Msg {
    Compose,
}

pub struct App;

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Compose => {
                let navigator = ctx.link().navigator().unwrap();
                wasm_bindgen_futures::spawn_local(async move {
                    let response = reqwasm::http::Request::get("/post/new").send().await.unwrap();
                    let json: blog_common::dto::Response<u64> = response.json().await.unwrap();
                    if json.status == 0 {
                        navigator.push(Route::ComposePost { id: json.data.unwrap() });
                        // yew_router::push_route(crate::router::Route::ComposePost { id: json.data.unwrap() });
                    } else {
                        // ctx.link().location().unwrap().route().set_href("/management");
                        if let Some(loc) = web_sys::window().map(|window| window.location()) {
                            let _ = loc.set_href("/management");
                        } else {
                            console_log!("get location failed");
                        }
                    }
                });
            },
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            // https://cn.bing.com/search?form=MOZLBR&pc=MOZI&q=free+blog+logo
            // https://www.designevo.com/logo-maker/
            <>
                <nav class="navbar" role="navigation" aria-label="main navigation">
                  <div class="navbar-brand">
                    <a class="navbar-item" href="/">
                      <img src="/asset/logo.png" width="115" height="32"/>
                    </a>

                    <a role="button" class="navbar-burger" aria-label="menu" aria-expanded="false" data-target="moreNavs">
                      <span aria-hidden="true"></span>
                      <span aria-hidden="true"></span>
                      <span aria-hidden="true"></span>
                    </a>
                  </div>

                  <div id="moreNavs" class="navbar-menu">
                    <div class="navbar-start">
                        <Link<Route> classes={"navbar-item"} to={Route::ListPosts}>
                            {"博客/Home"}
                        </Link<Route>>
                        <Link<Route> classes={"navbar-item"} to={Route::Tags}>
                            {"标签/Tags"}
                        </Link<Route>>

                      <div class="navbar-item has-dropdown is-hoverable">
                        <a class="navbar-link">
                          {"其它/More"}
                        </a>

                        <div class="navbar-dropdown">
                          <a class="navbar-item" href="/management">
                            {"管理/Management"}
                          </a>
                          <hr class="navbar-divider"/>
                            <Link<Route> classes={"navbar-item"} to={Route::About}>
                                {"关于/About"}
                            </Link<Route>>
                        </div>
                      </div>
                    </div>
                    <div class="navbar-end">
                      <div class="navbar-item">
                        <div class="buttons">
                            <button class="button" onclick={ctx.link().callback(|_| Msg::Compose)}>
                                <span class="icon">
                                    <i class="far fa-edit"></i>
                                </span>
                                <span>{"写博客/Compose"}</span>
                            </button>
                        </div>
                      </div>
                    </div>
                  </div>
                </nav>
                <main>
                    <Switch<Route> render={Switch::render(switch)} />
                </main>
                <footer class="footer">
                    <div class="content has-text-centered">
                        { "Powered by " }
                        <a href="https://yew.rs">{ "Yew" }</a>
                        { " using " }
                        <a href="https://bulma.io">{ "Bulma" }</a>
                        { " and images from " }
                        <a href="https://unsplash.com">{ "Unsplash" }</a>{" & "}<a href="https://picsum.photos">{ "Picsum" }</a>
                        {" Icons created by "}
                        <a href="https://www.flaticon.com/free-icons/blog">{"Freepik - Flaticon"}</a>
                    </div>
                    <div class="content has-text-centered">
                        { "Made by Songday with Love &hearts; ♥️" }
                    </div>
                </footer>
            </>
        }
    }
}
