use blog_common::dto::{post::PostDetail, tag::TagUsageAmount, PaginationData};
use yew::{
    agent::Bridged,
    html,
    services::{fetch::FetchTask, ConsoleService},
    Bridge, Callback, Component, ComponentLink, FocusEvent, Html, InputData, MouseEvent, Properties, ShouldRender,
};
use yew_router::{agent::RouteRequest::ChangeRoute, prelude::*};

use crate::{
    app::AppRoute,
    util::{request, store, Error},
    val,
};

pub(crate) struct Model {
    error: Option<Error>,
    fetch_task: Option<FetchTask>,
    post_response: Callback<Result<PaginationData<Vec<PostDetail>>, Error>>,
    tag_response: Callback<Result<Vec<TagUsageAmount>, Error>>,
    link: ComponentLink<Self>,
    posts: Vec<PostDetail>,
    tags: Vec<TagUsageAmount>,
}

pub(crate) enum Msg {
    PostListResponse(Result<PaginationData<Vec<PostDetail>>, Error>),
    TopTagsResponse(Result<Vec<TagUsageAmount>, Error>),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            error: None,
            fetch_task: None,
            post_response: link.callback(Msg::PostListResponse),
            tag_response: link.callback(Msg::TopTagsResponse),
            link,
            posts: vec![],
            tags: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::PostListResponse(Ok::<PaginationData<Vec<PostDetail>>, _>(d)) => {
                ConsoleService::log("Got post list data");
                self.posts = d.data;
                let r = request::get(val::TOP_TAG_URI, self.link.callback(Msg::TopTagsResponse));
                self.fetch_task = Some(r);
                return true;
            },
            Msg::PostListResponse(Err::<_, Error>(err)) => {
                eprintln!("{:?}", err);
                ConsoleService::log(&format!("{}", &err));
                self.error = Some(err);
                self.fetch_task = None;
                return true;
            },
            Msg::TopTagsResponse(Ok::<Vec<TagUsageAmount>, _>(tags)) => {
                self.tags = tags;
                self.fetch_task = None;
                return true;
            },
            Msg::TopTagsResponse(Err::<_, Error>(err)) => {
                eprintln!("{:?}", err);
                ConsoleService::log(&format!("{}", &err));
                self.error = Some(err);
                self.fetch_task = None;
                return true;
            },
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender { false }

    fn view(&self) -> Html {
        html! {
            <div class="row">
                <div class="col-9">
                <p class="fs-3">{"最新博客"}</p>
                {
                    for self.posts.iter().map(|b| {
                        html! {
                        <>
                        <div class="row"><div class="col" style="font-size:150%">
                            <i class="bi bi-journal-text"></i>
                            <RouterAnchor<AppRoute> route=AppRoute::PostShow({b.id})> {&b.title} </RouterAnchor<AppRoute>>
                        </div></div>
                        <div class="row"><div class="col">{&b.content}</div></div>
                        <div class="row"><div class="col" style="font-size:80%;font-color:gray">{&b.created_at}</div></div>
                        <div class="row"><div class="col">
                        {
                            if b.tags.is_some() {
                                html! {
                                <>
                                    <i class="bi bi-tags"></i>
                                    {
                                        for b.tags.as_ref().unwrap().iter().map(|t| {
                                            html! {
                                                <RouterAnchor<AppRoute> route=AppRoute::PostListByTag(t.to_string(), 1) classes="link-success ms-1"> {t} </RouterAnchor<AppRoute>>
                                            }
                                        })
                                    }
                                </>
                                }
                            } else {
                                html!{}
                            }
                        }
                        </div></div>
                        <div class="row"><div class="col pb-3"></div></div>
                        </>
                        }
                    })
                }
                </div>
                <div class="col-3">
                    <p class="fs-3">{"热门标签"}</p>
                    <div class="list-group">
                    {
                        for self.tags.iter().map(|t| {
                            html! {
                            <>
                                <RouterAnchor<AppRoute> route=AppRoute::PostListByTag((&t.name).to_string(), 1) classes="list-group-item list-group-item-action">
                                    <div class="d-flex w-100 justify-content-between">
                                      <h6 class="mb-1"><i class="bi bi-tag"></i> {&t.name}</h6>
                                      <span class="badge bg-primary rounded-pill">{&t.amount}</span>
                                    </div>
                                </RouterAnchor<AppRoute>>
                            </>
                            }
                        })
                    }
                    </div>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let url = format!("{}1", val::BLOG_LIST_URI);
            let r = request::get(&url, self.link.callback(Msg::PostListResponse));
            self.fetch_task = Some(r);
        }
    }
}
