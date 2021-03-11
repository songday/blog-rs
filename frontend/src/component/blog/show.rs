use alloc::{string::String, vec::Vec};
use core::{convert::Into, iter::IntoIterator};

use yew::{
    html, html::NodeRef, services::fetch::FetchTask, Callback, Component, ComponentLink, FocusEvent, Html, InputData,
    Properties, ShouldRender,
};
use yew_router::prelude::*;

use blog_common::dto::post::PostDetail;

use crate::{
    app::AppRoute,
    component::{self, error::ShowErrors},
    util::{request, Error},
    val,
};

pub(crate) struct Model {
    blog_id: i64,
    blog_detail: Option<PostDetail>,
    error: Option<Error>,
    fetch_task: Option<FetchTask>,
    response: Callback<Result<PostDetail, Error>>,
    // node_ref: NodeRef,
}

pub(crate) enum Msg {
    Response(Result<PostDetail, Error>),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub blog_id: i64,
}

impl Model {
    /// Dangerously set innerHTML for article body
    fn view_body(&self) -> Html { crate::component::raw_html("div", &self.blog_detail.as_ref().unwrap().content) }
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            blog_id: props.blog_id,
            blog_detail: None,
            error: None,
            fetch_task: None,
            response: link.callback(Msg::Response),
            // node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Response(Ok::<PostDetail, _>(blog)) => {
                // let html = markdown_to_html(&blog.content, &ComrakOptions::default());
                // let el = self.node_ref.cast::<web_sys::Element>().unwrap();
                // el.set_inner_html(&html);
                self.blog_detail = Some(blog);
                self.fetch_task = None;
            },
            Msg::Response(Err::<_, Error>(err)) => {
                self.error = Some(err);
                self.fetch_task = None;
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.blog_id != props.blog_id {
            self.blog_id = props.blog_id;
            return true;
        }
        false
    }

    fn view(&self) -> Html {
        if self.blog_detail.is_some() {
            let blog = self.blog_detail.as_ref().unwrap();
            html! {
                <div>
                    <h1>{&blog.title}</h1>
                    // <div ref=self.node_ref.clone()/>
                    <div>{ self.view_body() }</div>
                    {
                        if blog.tags.is_some() {
                            html!{
                                <div>
                                    {"标签: "}
                                    {
                                        for blog.tags.as_ref().unwrap().iter().map(|t| {
                                            html! {
                                                <RouterAnchor<AppRoute> route=AppRoute::BlogListByTag(t.to_string(), 1) classes="tag-link"> {t} </RouterAnchor<AppRoute>>
                                            }
                                        })
                                    }
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            }
        } else {
            html! {
                <div>{"正在载入，很快的..."}</div>
            }
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render && self.blog_detail.is_none() && self.blog_id > 0 {
            let mut url = String::with_capacity(64);
            url.push_str(val::BLOG_SHOW_URL);
            url.push_str(self.blog_id.to_string().as_str());
            let fetch_task = request::get::<PostDetail>(url.as_str(), self.response.clone());
            self.fetch_task = Some(fetch_task);
        }
    }
}
