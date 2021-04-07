use alloc::{boxed::Box, format, string::String, vec::Vec};
use core::iter::Iterator;

use yew::{
    agent::Bridged,
    html,
    services::{fetch::FetchTask, ConsoleService},
    Bridge, Callback, Component, ComponentLink, FocusEvent, Html, InputData, MouseEvent, Properties, ShouldRender,
};
use yew_router::{agent::RouteRequest::ChangeRoute, prelude::*};

use blog_common::dto::{
    post::{NewPost, PostDetail},
    PaginationData,
};

use crate::{
    app::AppRoute,
    component::error::ShowErrors,
    util::{request, Error},
    val,
};

#[derive(Properties, Clone)]
pub(crate) struct Props {
    pub tag: Option<String>,
    pub current_page: u8,
}

pub(crate) struct Model {
    props: Props,
    error: Option<Error>,
    fetch_task: Option<FetchTask>,
    response: Callback<Result<PaginationData<Vec<PostDetail>>, Error>>,
    router_agent: Box<dyn Bridge<RouteAgent>>,
    link: ComponentLink<Self>,
    blogs: Vec<PostDetail>,
    total_page_num: u8,
}

pub(crate) enum Msg {
    Ignore,
    Request,
    Response(Result<PaginationData<Vec<PostDetail>>, Error>),
    PaginationChanged(u8),
}

impl Model {
    fn request(&mut self) {
        let mut url = String::with_capacity(64);
        if self.props.tag.is_some() {
            url.push_str(val::BLOG_TAG_LIST_URL);
            url.push_str(self.props.tag.as_ref().unwrap());
            url.push('/');
        } else {
            url.push_str(val::BLOG_LIST_URL);
        }
        url.push_str(self.props.current_page.to_string().as_str());

        let fetch_task = request::get::<PaginationData<Vec<PostDetail>>>(&url, self.response.clone());
        self.fetch_task = Some(fetch_task);
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            error: None,
            fetch_task: None,
            // response: Default::default(),
            response: link.callback(Msg::Response),
            router_agent: RouteAgent::bridge(link.callback(|_| Msg::Ignore)),
            link,
            blogs: Vec::new(),
            total_page_num: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => {},
            Msg::Request => {
                self.request();
            },
            Msg::Response(Ok::<PaginationData<Vec<PostDetail>>, _>(blog)) => {
                self.blogs = blog.data;
                self.total_page_num = (blog.total / 20) as u8;
                if blog.total % 20 != 0 {
                    self.total_page_num = self.total_page_num + 1;
                }
                self.fetch_task = None;
                return true;
            },
            Msg::Response(Err::<_, Error>(err)) => {
                ConsoleService::log(&format!("{}", &err));
                self.error = Some(err);
                self.fetch_task = None;
                return true;
            },
            Msg::PaginationChanged(p) => {
                if self.props.tag.is_none() {
                    self.router_agent.send(ChangeRoute(AppRoute::BlogList(p).into()));
                } else {
                    let tag = String::from(self.props.tag.as_ref().unwrap());
                    self.router_agent
                        .send(ChangeRoute(AppRoute::BlogListByTag(tag, p).into()));
                }
                self.props.current_page = p;
                self.request();
            },
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if (self.props.tag.is_some() && props.tag.is_none())
            || (self.props.tag.is_none() && props.tag.is_some())
            || (self.props.tag.is_some()
                && props.tag.is_some()
                && !self.props.tag.as_ref().unwrap().eq(props.tag.as_ref().unwrap()))
        {
            self.props = props;
            return true;
        }
        false
    }

    fn view(&self) -> Html {
        let mut pages: Vec<u8> = vec![];
        for page in 0..self.total_page_num {
            pages.push(page + 1);
        }

        html! {
            <>
            {
                for self.blogs.iter().map(|b| {
                    html! {
                    <>
                    <div>
                        <RouterAnchor<AppRoute> route=AppRoute::BlogShow({b.id})> {&b.title} </RouterAnchor<AppRoute>>
                    </div>
                    <div>{&b.content}</div>
                    <div>{&b.created_at}</div>
                    <div>
                    {
                        if b.tags.is_some() {
                            html! {
                                for b.tags.as_ref().unwrap().iter().map(|t| {
                                    html! {
                                        <button type="button" class="btn btn-link btn-sm ms-1">{t}</button>
                                    }
                                })
                            }
                        } else {
                            html!{}
                        }
                    }
                    </div>
                    <hr/>
                    </>
                    }
                })
            }
            <br/>
            <nav aria-label="Page navigation example">
                <ul class="pagination">
                {
                    for pages.iter().map(|page| {
                        let is_current = page == &self.props.current_page;
                        let page_item_class = if is_current {
                            "page-item active"
                        } else {
                            "page-item"
                        };
                        let page = page.clone();
                        let onclick = self.link.callback(move |ev: MouseEvent| {ev.prevent_default(); Msg::PaginationChanged(page)});
                        html! {
                            <li class=page_item_class>
                                <a class="page-link" onclick=onclick>{page}</a>
                            </li>
                        }
                    })
                }
                </ul>
            </nav>
            </>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.request();
            // let loc = web_sys::window().unwrap().location();
            // let hash = loc.hash().unwrap();
        }
    }
}
