use alloc::{rc::Rc, string::String};

use blog_common::{
    dto::{
        management::{Setting, SiteData},
        user::{UserInfo, UserInfoWrapper},
    },
    val as CommonVal,
};
use wasm_bindgen::JsCast;
use yew::{
    prelude::*,
    services::{fetch::FetchTask, ConsoleService},
};
use yew_router::{
    prelude::*,
    switch::{AllowMissing, Permissive},
    Switch,
};

use crate::{
    component::{
        about, contribute,
        header::{nav, user},
        index,
        post::{compose, list, show},
        tag::top,
        user::{login, register},
    },
    util::{request, store, Error},
    val,
};

#[derive(Switch, Clone)]
pub(crate) enum AppRoute {
    #[to = "/#/about"]
    About,
    #[to = "/#/contribute"]
    Contribute,
    #[to = "/#/post/compose"]
    PostCompose,
    #[to = "/#/post/list/{page}"]
    PostList(u8),
    #[to = "/#/post/tag/{tag}/{page}"]
    PostListByTag(String, u8),
    #[to = "/#/post/show/{id}"]
    PostShow(i64),
    #[to = "/#/post/edit/{id}"]
    PostEdit(i64),
    // #[to = "/#/post/upload"]
    // BlogUpload,
    #[to = "/#/tag/top"]
    TopTags,
    #[to = "/#/user/login"]
    UserLogin,
    #[to = "/#/user/register"]
    UserRegister,
    #[to = "/#/user/logout"]
    UserLogout,
    #[to = "/#/page-not-found"]
    PageNotFound(Permissive<String>),
    #[to = "/"]
    Home,
}

pub enum Msg {
    Authenticated(UserInfo),
    Logout,
    LogoutResponse(Result<String, Error>),
    SiteDataResponse(Result<SiteData, Error>),
}

pub(crate) struct Model {
    pub settings: Setting,
    pub user: Option<UserInfo>,
    fetch_task: Option<FetchTask>,
    pub link: ComponentLink<Self>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            settings: Setting::default(),
            user: None,
            fetch_task: None,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Authenticated(user) => {
                ConsoleService::log("signed in");
                // store::save(CommonVal::SESSION_ID_HEADER_NAME, Some(user.access_token));
                // ConsoleService::log("saved auth to store");
                self.user = Some(user);
                return true;
            },
            Msg::Logout => {
                ConsoleService::log("to sign out");
                let r = request::get(val::USER_LOGOUT_URI, self.link.callback(Msg::LogoutResponse));
                self.fetch_task = Some(r);
            },
            Msg::LogoutResponse(Ok::<String, _>(s)) => {
                store::save(CommonVal::SESSION_ID_HEADER_NAME, None);
                ConsoleService::log("signed out");
                self.user = None;
                self.fetch_task = None;
                return true;
            },
            Msg::LogoutResponse(Err::<_, Error>(e)) => {},
            Msg::SiteDataResponse(Ok::<SiteData, _>(site_data)) => {
                ConsoleService::log("Got site data");
                // ConsoleService::log("saved auth to store");
                self.settings = site_data.settings;
                self.user = site_data.user_info;
                ConsoleService::log("Set site data");
                return true;
            },
            Msg::SiteDataResponse(Err::<_, Error>(e)) => {
                eprintln!("{:?}", e);
            },
        }
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender { false }

    fn view(&self) -> Html {
        // let callback_login = self.link.callback(Msg::Authenticated);
        let auth_callback = Rc::new(self.link.callback(Msg::Authenticated));
        // let callback_logout = self.link.callback(|_| Msg::Logout);
        let logout_callback = Rc::new(self.link.callback(|_| Msg::Logout));
        // let none_tag: Option<String> = None;
        let user = self.user.clone();

        html! {
            <div class="container">
                <div class="row">
                    <div class="col">
                        {&self.settings.name}
                        {" | "}
                        // <RouterAnchor<AppRoute> route=AppRoute::TopTags> {"热门标签"} </RouterAnchor<AppRoute>>{" | "}
                        <RouterAnchor<AppRoute> route=AppRoute::Home> {"首页"} </RouterAnchor<AppRoute>>{" | "}
                        <RouterAnchor<AppRoute> route=AppRoute::PostList(1)> {"全部博客"} </RouterAnchor<AppRoute>>
                    </div>
                    <div class="col text-end">
                        <user::Model user=&self.user callback=logout_callback.clone()/>
                    </div>
                </div>
                <div class="row">
                    <div class="col pb-3"></div>
                </div>
                <div class="row">
                    <div class="col">
                        <Router<AppRoute>
                            render = Router::render(move |switch: AppRoute| {
                                match switch {
                                    AppRoute::Home => html!{<index::Model/>},
                                    AppRoute::About => html!{<about::Model/>},
                                    AppRoute::Contribute => html!{<contribute::Model/>},
                                    AppRoute::UserLogin => html!{<login::Model callback=auth_callback.clone()/>},
                                    AppRoute::UserRegister => html!{<register::Model/>},
                                    AppRoute::PostList(page) => html!{<list::Model tag=None::<String> current_page=page/>},
                                    AppRoute::PostListByTag(tag, page) => html!{<list::Model tag=tag current_page=page/>},
                                    AppRoute::PostCompose => html!{<compose::Model blog_id=None::<i64> user_info=user.clone()/>},
                                    AppRoute::PostShow(id) => html!{<show::Model blog_id=id/>},
                                    AppRoute::PostEdit(id) => html!{<compose::Model blog_id=id user_info=user.clone()/>},
                                    // AppRoute::BlogUpload => html!{<upload::Model/>},
                                    AppRoute::TopTags => html!{<top::Model/>},
                                    _ => html!{"Page not found :("},
                                }
                            })
                            redirect = Router::redirect(|route: Route| {
                                AppRoute::PageNotFound(Permissive(Some(route.route)))
                            })
                        />
                    </div>
                </div>
                <hr/>
                <div class="row">
                    <div class="col">
                        <RouterAnchor<AppRoute> route=AppRoute::Contribute> {"欢迎投稿"} </RouterAnchor<AppRoute>>
                        <br/>
                        <RouterAnchor<AppRoute> route=AppRoute::About> {"关于我们"} </RouterAnchor<AppRoute>>
                    </div>
                    <div class="col text-end">
                        { crate::component::raw_html("span", "&copy; 2021") }
                    </div>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        // Get current user info if a token is available when mounted
        if first_render {
            // 由于使用了 HttpOnly，所以JS读不了cookie，就需要读取 local storage
            // local storage
            // let token = store::get(CommonVal::USER_AUTH_MARK_HEADER);
            // if token.is_some() {
            //     // let token = token.as_ref().unwrap();
            //     let r = request::get(val::USER_INFO_URI, self.link.callback(Msg::SiteDataResponse));
            //     self.fetch_task = Some(r);
            // }
            // cookie
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let html_document = document.dyn_into::<web_sys::HtmlDocument>().unwrap();
            let cookie: Option<String> = match html_document.cookie() {
                Ok(c) => {
                    ConsoleService::log(&c);
                    Some(c)
                },
                Err(e) => {
                    ConsoleService::log(&format!("{:?}", e));
                    None
                },
            };
            if cookie.is_none() {
                return;
            }
            let cookie = cookie.unwrap();
            let p = cookie.find(CommonVal::USER_AUTH_MARK_HEADER);
            if p.is_none() {
                return;
            }
            // let p = p.unwrap() + search.len();
            // let token = match cookie[p..].find(';').map(|i| i + p) {
            //     Some(semicolon) => &cookie[p..semicolon],
            //     None => &cookie[p..],
            // };
            let r = request::get(val::SITE_DATA_URI, self.link.callback(Msg::SiteDataResponse));
            self.fetch_task = Some(r);
        }
    }
}
