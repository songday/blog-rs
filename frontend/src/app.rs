use alloc::{rc::Rc, string::String};

use yew::{
    prelude::*,
    services::{fetch::FetchTask, ConsoleService},
};
use yew_router::{
    prelude::*,
    switch::{AllowMissing, Permissive},
    Switch,
};

use blog_common::{
    dto::user::{UserInfo, UserInfoWrapper},
    val as CommonVal,
};

use crate::{
    component::{
        about,
        blog::{compose, list, show, upload},
        contribute,
        header::{nav, user},
        index,
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
    #[to = "/#/blog/compose"]
    BlogCompose,
    #[to = "/#/blog/list/{page}"]
    BlogList(u8),
    #[to = "/#/blog/tag/{tag}/{page}"]
    BlogListByTag(String, u8),
    #[to = "/#/blog/show/{id}"]
    BlogShow(i64),
    #[to = "/#/blog/upload"]
    BlogUpload,
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
    Authenticated(UserInfoWrapper),
    Logout,
    LogoutResponse(Result<String, Error>),
    UserInfoResponse(Result<UserInfo, Error>),
}

pub(crate) struct Model {
    pub user: Option<UserInfo>,
    fetch_task: Option<FetchTask>,
    pub link: ComponentLink<Self>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            user: None,
            fetch_task: None,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Authenticated(user) => {
                ConsoleService::log("signed in");
                store::save(CommonVar::AUTH_HEADER_NAME, Some(user.access_token));
                ConsoleService::log("saved auth to store");
                self.user = Some(user.user_info);
                return true;
            },
            Msg::Logout => {
                ConsoleService::log("to sign out");
                let r = request::get(val::USER_LOGOUT_URL, self.link.callback(Msg::LogoutResponse));
                self.fetch_task = Some(r);
            },
            Msg::LogoutResponse(Ok::<String, _>(s)) => {
                store::save(CommonVar::AUTH_HEADER_NAME, None);
                ConsoleService::log("signed out");
                self.user = None;
                self.fetch_task = None;
                return true;
            },
            Msg::LogoutResponse(Err::<_, Error>(e)) => {},
            Msg::UserInfoResponse(Ok::<UserInfo, _>(user)) => {
                ConsoleService::log("signed in");
                ConsoleService::log("saved auth to store");
                self.user = Some(user);
                return true;
            },
            Msg::UserInfoResponse(Err::<_, Error>(e)) => {},
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

        html! {
            <>
                <div class="pure-g">
                    <div class="pure-u-1-2"><p>{"70年代、80年代、90年代"}</p></div>
                    <div class="pure-u-1-2 tar">
                    <p>
                        <RouterAnchor<AppRoute> route=AppRoute::Home> {"热门标签"} </RouterAnchor<AppRoute>>{" | "}
                        <RouterAnchor<AppRoute> route=AppRoute::BlogList(1)> {"全部内容"} </RouterAnchor<AppRoute>>
                    </p>
                    </div>
                </div>
                <div class="pure-g">
                    <div class="pure-u-1-1">
                        <Router<AppRoute>
                            render = Router::render(move |switch: AppRoute| {
                                match switch {
                                    AppRoute::Home => html!{<index::Model/>},
                                    AppRoute::About => html!{<about::Model/>},
                                    AppRoute::Contribute => html!{<contribute::Model/>},
                                    AppRoute::UserLogin => html!{<login::Model callback=auth_callback.clone()/>},
                                    AppRoute::UserRegister => html!{<register::Model/>},
                                    AppRoute::BlogList(page) => html!{<list::Model tag=None::<String> current_page=page/>},
                                    AppRoute::BlogListByTag(tag, page) => html!{<list::Model tag=tag current_page=page/>},
                                    AppRoute::BlogCompose => html!{<compose::Model/>},
                                    AppRoute::BlogShow(id) => html!{<show::Model blog_id=id/>},
                                    AppRoute::BlogUpload => html!{<upload::Model/>},
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
                <div class="pure-g">
                    <div class="pure-u-1-2">
                        <RouterAnchor<AppRoute> route=AppRoute::Contribute> {"欢迎投稿"} </RouterAnchor<AppRoute>>
                        <br/>
                        <RouterAnchor<AppRoute> route=AppRoute::About> {"关于我们"} </RouterAnchor<AppRoute>>
                    </div>
                    <div class="pure-u-1-2 tar">
                        <user::Model user=&self.user callback=logout_callback.clone()/>
                    </div>
                </div>
                <div class="pure-g">
                    <div class="pure-u-1-1 tac">
                        { crate::component::raw_html("span", "&copy; 2020.") }
                    </div>
                </div>
            </>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        // Get current user info if a token is available when mounted
        if first_render {
            let token = store::get(CommonVar::AUTH_HEADER_NAME);
            if token.is_some() {
                // let token = token.as_ref().unwrap();
                let r = request::get(val::USER_INFO_URL, self.link.callback(Msg::UserInfoResponse));
                self.fetch_task = Some(r);
            }
            // 由于使用了 HttpOnly，所以JS读不了cookie
            // let window = web_sys::window().unwrap();
            // let document = window.document().unwrap();
            // let html_document = document.dyn_into::<web_sys::HtmlDocument>().unwrap();
            // match html_document.cookie() {
            //     Ok(s) => ConsoleService::log(&s),
            //     Err(e) => ConsoleService::log(&format!("{:?}", e)),
            // };
            // let task = self.auth.current(self.current_user_response.clone());
            // self.current_user_task = Some(task);
        }
    }
}
