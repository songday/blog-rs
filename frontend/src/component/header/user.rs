use alloc::rc::Rc;

use yew::{prelude::*, virtual_dom::VNode};
use yew_router::prelude::*;

use blog_common::dto::user::UserInfo;

use crate::{
    app::AppRoute,
    util::{request, Error},
    val,
};

pub(crate) enum Msg {
    Logout,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub user: Option<UserInfo>,
    pub callback: Rc<Callback<()>>,
    // pub callback: Arc<Callback<()>>,
}

pub(crate) struct Model {
    props: Props,
    link: ComponentLink<Self>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self { Model { props, link } }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Logout => {
                self.props.callback.emit(());
            },
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props.user.is_none() && props.user.is_some() {
            self.props = props;
            return true;
        }
        if self.props.user.is_some() && props.user.is_none() {
            self.props = props;
            return true;
        }
        self.props.user.as_ref().unwrap().id != props.user.as_ref().unwrap().id
    }

    fn view(&self) -> VNode {
        html! {
            {
                if self.props.user.is_none() {
                    html! {
                    <>
                        <div><RouterAnchor<AppRoute> route=AppRoute::UserLogin> {"登录"} </RouterAnchor<AppRoute>></div>
                        <div><RouterAnchor<AppRoute> route=AppRoute::UserRegister> {"注册"} </RouterAnchor<AppRoute>></div>
                    </>
                    }
                } else {
                    html! {
                    <>
                        <div>{self.props.user.as_ref().unwrap().email.as_str()}</div>
                        <div><RouterAnchor<AppRoute> route=AppRoute::BlogCompose> {"新建内容"} </RouterAnchor<AppRoute>></div>
                        <div><a href="#logout" onclick=self.link.callback(|_| Msg::Logout)> { "退出" } </a></div>
                    </>
                    }
                }
            }
        }
    }
}
