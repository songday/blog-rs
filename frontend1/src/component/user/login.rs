use alloc::{boxed::Box, rc::Rc, string::String, vec::Vec};
use core::{convert::Into, default::Default, option::Option};

use blog_common::dto::{management::AdminUser, user::UserInfo};
use yew::{
    agent::Bridged, Context, html, Bridge, Callback, Component, ComponentLink, FocusEvent, Html,
    InputData, Properties,
};
use yew_router::{agent::RouteRequest::ChangeRoute, prelude::*};

use crate::{
    app::AppRoute,
    component::error::ShowErrors,
    util::{request, Error},
    val,
};

#[derive(PartialEq, Properties, Clone)]
pub struct Props {
    /// Callback when user is logged in successfully
    pub callback: Rc<Callback<UserInfo>>,
    // pub callback: Arc<Callback<UserInfo>>,
}

pub(crate) struct Model {
    login_params: AdminUser,
    error: Option<Error>,
    response: Callback<Result<UserInfo, Error>>,
    router_agent: Box<dyn Bridge<RouteAgent>>,
    props: Props,
}

pub(crate) enum Msg {
    Ignore,
    UpdateEmail(String),
    UpdatePassword(String),
    UpdateCaptcha(String),
    Request,
    Response(Result<UserInfo, Error>),
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;
    fn create(ctx: &Context<Self>) -> Self {
        Model {
            login_params: AdminUser::default(),
            error: None,
            response: ctx.link().callback(Msg::Response),
            router_agent: RouteAgent::bridge(ctx.link().callback(|_| Msg::Ignore)),
            props,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => {},
            Msg::UpdateEmail(s) => self.login_params.email = s,
            Msg::UpdatePassword(s) => self.login_params.password = s,
            Msg::UpdateCaptcha(s) => self.login_params.captcha = s,
            Msg::Request => {
                let fetch_task = request::post::<AdminUser, UserInfo>(
                    val::MANAGEMENT_LOGIN_URI,
                    self.login_params.clone(),
                    self.response.clone(),
                );
                self.fetch_task = Some(fetch_task);
            },
            Msg::Response(Ok::<UserInfo, _>(user)) => {
                self.fetch_task = None;
                self.props.callback.emit(user);
                self.router_agent.send(ChangeRoute(AppRoute::PostCompose.into()));
            },
            Msg::Response(Err::<_, Error>(err)) => {
                self.error = Some(err);
                self.fetch_task = None;
                return true;
            },
        }
        false
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> ShouldRender { false }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                // <RouterAnchor<AppRoute> route=AppRoute::UserRegister>
                //     { "Need an account?" }
                // </RouterAnchor<AppRoute>>
                <div class="row">
                    <div class="col">
                        <h1>{"登录"}</h1>
                    </div>
                </div>
                <div class="row">
                    <div class="col-md"></div>
                    <div class="col-md">
                        <div id="errorMessage" style="color:red;display:none"></div>
                        <form class="row g-3" onsubmit={self.link.callback(|ev: FocusEvent| {
                            ev.prevent_default();
                            Msg::Request
                        })}>
                            <div class="col-12">
                                <label for="aligned-email" class="form-label">{"邮箱地址"}</label>
                                <input type="email" id="aligned-email" class="form-control" placeholder="邮箱地址" oninput={self.link.callback(|e: InputData| Msg::UpdateEmail(e.value))}/>
                            </div>
                            <div class="col-12">
                                <label for="aligned-password" class="form-label">{"登录密码"}</label>
                                <input type="password" id="aligned-password" class="form-control" placeholder="密码" oninput={self.link.callback(|e: InputData| Msg::UpdatePassword(e.value))}/>
                            </div>
                            <div class="col-md-4">
                                <label for="aligned-captcha" class="form-label">{"验证码"}</label>
                                <input type="text" id="aligned-captcha" class="form-control" placeholder="验证码" oninput={self.link.callback(|e: InputData| Msg::UpdateCaptcha(e.value))}/>
                                <img src="/tool/verify-image"/>
                            </div>
                            <div class="col-12">
                                <button type="submit" class="btn btn-primary">{"登录"}</button>
                            </div>
                        </form>
                    </div>
                    <div class="col-md"></div>
                </div>
            </>
        }
    }
}
