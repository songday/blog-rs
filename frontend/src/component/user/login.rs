use alloc::{boxed::Box, rc::Rc, string::String, vec::Vec};
use core::{convert::Into, default::Default, option::Option};

use yew::{
    agent::Bridged, html, services::fetch::FetchTask, Bridge, Callback, Component, ComponentLink, FocusEvent, Html,
    InputData, Properties, ShouldRender,
};
use yew_router::{agent::RouteRequest::ChangeRoute, prelude::*};

use blog_common::dto::user::{UserInfoWrapper, UserParams};

use crate::{
    app::AppRoute,
    component::error::ShowErrors,
    util::{request, Error},
    val,
};

#[derive(PartialEq, Properties, Clone)]
pub struct Props {
    /// Callback when user is logged in successfully
    pub callback: Rc<Callback<UserInfoWrapper>>,
    // pub callback: Arc<Callback<UserInfoWrapper>>,
}

pub(crate) struct Model {
    login_params: UserParams,
    error: Option<Error>,
    fetch_task: Option<FetchTask>,
    response: Callback<Result<UserInfoWrapper, Error>>,
    router_agent: Box<dyn Bridge<RouteAgent>>,
    props: Props,
    link: ComponentLink<Self>,
}

pub(crate) enum Msg {
    Ignore,
    UpdateEmail(String),
    UpdatePassword(String),
    UpdateCaptcha(String),
    Request,
    Response(Result<UserInfoWrapper, Error>),
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model {
            login_params: UserParams::default(),
            error: None,
            fetch_task: None,
            response: link.callback(Msg::Response),
            router_agent: RouteAgent::bridge(link.callback(|_| Msg::Ignore)),
            props,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => {},
            Msg::UpdateEmail(s) => self.login_params.email = s,
            Msg::UpdatePassword(s) => self.login_params.password1 = s,
            Msg::UpdateCaptcha(s) => self.login_params.captcha = s,
            Msg::Request => {
                let fetch_task = request::post::<UserParams, UserInfoWrapper>(
                    val::USER_LOGIN_URL,
                    self.login_params.clone(),
                    self.response.clone(),
                );
                self.fetch_task = Some(fetch_task);
            },
            Msg::Response(Ok::<UserInfoWrapper, _>(user)) => {
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

    fn change(&mut self, _props: Self::Properties) -> ShouldRender { false }

    fn view(&self) -> Html {
        html! {
            <div>
                // <RouterAnchor<AppRoute> route=AppRoute::UserRegister>
                //     { "Need an account?" }
                // </RouterAnchor<AppRoute>>
                <ShowErrors error=&self.error />
                <form class="pure-form pure-form-stacked" onsubmit=self.link.callback(|ev: FocusEvent| {
                    ev.prevent_default();
                    Msg::Request
                })>
                    <fieldset>
                        <legend>{"Sign In"}</legend>
                        // <label for="stacked-email">{"Email"}</label>
                        <input
                            id="stacked-email"
                            type="email"
                            placeholder="Email" required=true
                            value=&self.login_params.email
                            oninput=self.link.callback(|e: InputData| Msg::UpdateEmail(e.value))
                            />
                        // <label for="stacked-password">{"Password"}</label>
                        <input
                            id="stacked-password"
                            type="password"
                            placeholder="Password" required=true
                            value=&self.login_params.password1
                            oninput=self.link.callback(|e: InputData| Msg::UpdatePassword(e.value))
                            />
                        // <label for="stacked-captcha">{"Captcha"}</label>
                        <img src=val::VERIFY_IMAGE_URL />
                        <input
                            id="stacked-captcha"
                            type="text"
                            placeholder="Captcha" required=true
                            value=&self.login_params.captcha
                            oninput=self.link.callback(|e: InputData| Msg::UpdateCaptcha(e.value))
                            />
                        <button
                            class="pure-button pure-button-primary"
                            type="submit">
                            { "Sign in" }
                        </button>
                    </fieldset>
                </form>
            </div>
        }
    }
}
