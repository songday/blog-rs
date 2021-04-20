use alloc::{boxed::Box, format, string::String, vec::Vec};
use core::iter::Iterator;

use blog_common::dto::tag::TagUsageAmount;
use yew::{
    agent::Bridged,
    html,
    services::{fetch::FetchTask, ConsoleService},
    Bridge, Callback, Component, ComponentLink, FocusEvent, Html, InputData, MouseEvent, Properties, ShouldRender,
};
use yew_router::{agent::RouteRequest::ChangeRoute, prelude::*};

use crate::{
    app::AppRoute,
    component::error::ShowErrors,
    util::{request, Error},
    val,
};

pub(crate) struct Model {
    error: Option<Error>,
    fetch_task: Option<FetchTask>,
    response: Callback<Result<Vec<TagUsageAmount>, Error>>,
    router_agent: Box<dyn Bridge<RouteAgent>>,
    link: ComponentLink<Self>,
    tags: Vec<TagUsageAmount>,
    total_page_num: u8,
}

pub(crate) enum Msg {
    Ignore,
    Request,
    Response(Result<Vec<TagUsageAmount>, Error>),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            error: None,
            fetch_task: None,
            // response: Default::default(),
            response: link.callback(Msg::Response),
            router_agent: RouteAgent::bridge(link.callback(|_| Msg::Ignore)),
            link,
            tags: Vec::new(),
            total_page_num: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => {},
            Msg::Request => {
                // self.request();
            },
            Msg::Response(Ok::<Vec<TagUsageAmount>, _>(tags)) => {
                self.tags = tags;
                self.fetch_task = None;
                return true;
            },
            Msg::Response(Err::<_, Error>(err)) => {
                ConsoleService::log(&format!("{}", &err));
                self.error = Some(err);
                self.fetch_task = None;
                return true;
            },
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender { false }

    fn view(&self) -> Html {
        html! {
            <div class="row"><div class="col-5"><ul class="list-group list-group-numbered">
            {
                for self.tags.iter().map(|t| {
                    html! {
                    <li class="list-group-item d-flex justify-content-between align-items-start">
                        <i class="bi bi-tag"></i> <RouterAnchor<AppRoute> route=AppRoute::PostListByTag((&t.name).to_string(), 1)> {&t.name} </RouterAnchor<AppRoute>>
                        <span class="badge bg-primary rounded-pill">{&t.amount}</span>
                    </li>
                    }
                })
            }
            </ul></div></div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let task = request::get::<Vec<TagUsageAmount>>(val::TOP_TAG_URL, self.link.callback(Msg::Response));
            self.fetch_task = Some(task);
        }
    }
}
