use alloc::{boxed::Box, string::String};

use yew::{
    agent::Bridged, html, services::fetch::FetchTask, Bridge, Callback, Component, ComponentLink, FocusEvent, Html,
    InputData, ShouldRender,
};
use yew_router::{agent::RouteRequest::ChangeRoute, prelude::*};

use crate::{
    app::AppRoute,
    component::error::ShowErrors,
    util::{request, Error},
    val,
};
use blog_common::dto::user::{UserInfoWrapper, UserParams};

pub(crate) struct Model {
    register_params: UserParams,
    error: Option<Error>,
    fetch_task: Option<FetchTask>,
    response: Callback<Result<UserInfoWrapper, Error>>,
    router_agent: Box<dyn Bridge<RouteAgent>>,
    link: ComponentLink<Self>,
}

pub(crate) enum Msg {
    Ignore,
    UpdateEmail(String),
    UpdatePassword1(String),
    UpdatePassword2(String),
    UpdateCaptcha(String),
    Request,
    Response(Result<UserInfoWrapper, Error>),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model {
            register_params: UserParams::default(),
            error: None,
            fetch_task: None,
            response: link.callback(Msg::Response),
            router_agent: RouteAgent::bridge(link.callback(|_| Msg::Ignore)),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => {},
            Msg::UpdateEmail(s) => self.register_params.email = s,
            Msg::UpdatePassword1(s) => self.register_params.password1 = s,
            Msg::UpdatePassword2(s) => self.register_params.password2 = s,
            Msg::UpdateCaptcha(s) => self.register_params.captcha = s,
            Msg::Request => {
                let fetch_task = request::post::<UserParams, UserInfoWrapper>(
                    val::USER_REGISTER_URL,
                    self.register_params.clone(),
                    self.response.clone(),
                );
                self.fetch_task = Some(fetch_task);
            },
            Msg::Response(Ok::<UserInfoWrapper, _>(user)) => {
                self.fetch_task = None;
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
                // <RouterAnchor<AppRoute> route=AppRoute::UserLogin>
                //     { "Have an account already? Sign in!" }
                // </RouterAnchor<AppRoute>>
                <ShowErrors error=&self.error />
                <form class="pure-form pure-form-stacked" onsubmit=self.link.callback(|ev: FocusEvent| {
                    ev.prevent_default();
                    Msg::Request
                })>
                    <fieldset>
                        <legend>{"Sign Up"}</legend>
                        <input
                            class="form-control form-control-lg"
                            type="email"
                            placeholder="Email" required=true
                            value=&self.register_params.email
                            oninput=self.link.callback(|e: InputData| Msg::UpdateEmail(e.value))
                            />
                        <input
                            class="form-control form-control-lg"
                            type="password"
                            placeholder="Password" required=true
                            value=&self.register_params.password1
                            oninput=self.link.callback(|e: InputData| Msg::UpdatePassword1(e.value))
                            />
                        <input
                            class="form-control form-control-lg"
                            type="password"
                            placeholder="Same password again" required=true
                            value=&self.register_params.password2
                            oninput=self.link.callback(|e: InputData| Msg::UpdatePassword2(e.value))
                            />
                        <img src=val::VERIFY_IMAGE_URL />
                        <input
                            class="form-control form-control-lg"
                            type="text"
                            placeholder="Captcha" required=true
                            value=&self.register_params.captcha
                            oninput=self.link.callback(|e: InputData| Msg::UpdateCaptcha(e.value))
                            />
                        <button
                            class="pure-button pure-button-primary"
                            type="submit">
                            { "Sign up" }
                        </button>
                    </fieldset>
                </form>
            </div>
        }
    }
}
