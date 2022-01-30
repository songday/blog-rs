use yew::prelude::*;
use yew_router::prelude::*;

use crate::page::post::{PostCompose, PostDetail, PostsList, PostsListByTag};
use crate::page::tag::TagsList;

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/posts/:id")]
    ShowPost { id: u64 },
    #[at("/posts/compose/:id")]
    ComposePost { id: u64 },
    #[at("/posts/tag/:tag_name")]
    ListPostsByTag { tag_name: String },
    #[at("/tags")]
    Tags,
    #[at("/about")]
    About,
    #[at("/")]
    ListPosts,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(About)]
fn about() -> Html {
    html! {
        <div class="container content">
        <p>{"省资源"}</p>
        <ul>
        <li>{"文件小"}</li>
        <li>{"在Windows 10上，只占用了xxx内存"}</li>
        <li>{"无任何后台或定时任务"}</li>
        </ul>
        <p>{"速度快"}</p>
        <ul>
        <li>{"压测数据"}</li>
        <li>{"所有内嵌的静态文件，均使用gzip压缩，提高网络传输速度"}</li>
        </ul>
        <p>{"功能丰富"}</p>
        <ul>
        <li>{"集成 Markdown 编辑器"}</li>
        <li>{"导出博客数据"}</li>
        <li>{"支持提供单独的静态文件服务，并支持动态渲染 Markdown 文件（md格式）"}</li>
        </ul>
        <hr />
        <p>{"Light"}</p>
        <ul>
        <li>{"Small file size."}</li>
        <li>{"It consumes only xxxM on Windows 10."}</li>
        <li>{"No daemon service and schedule task."}</li>
        </ul>
        <p>{"Fast"}</p>
        <ul>
        <li>{"Some benchmark."}</li>
        <li>{"All embed files were gzipped for network transfer."}</li>
        </ul>
        <p>{"Features"}</p>
        <ul>
        <li>{"Markdown editor included."}</li>
        <li>{"Export posts data to other static site generators."}</li>
        <li>{"Run as a simple file server, and render markdown files (md ext) dynamically."}</li>
        </ul>
        </div>
    }
}

#[function_component(NotFound)]
fn not_found() -> Html {
    html! {
        <section class="hero is-danger is-bold is-large">
            <div class="hero-body">
                <div class="container">
                    <h1 class="title">
                        { "找不到请求的页面/Page not found" }
                    </h1>
                    <h2 class="subtitle">
                        { "找不到请求的页面/Page page does not seem to exist." }
                    </h2>
                </div>
            </div>
        </section>
    }
}

pub fn switch(routes: &Route) -> Html {
    match routes {
        Route::ShowPost { id } => {
            html! { <PostDetail post_id={*id} /> }
        },
        Route::ListPostsByTag { tag_name } => {
            html! { <PostsListByTag tag_name={String::from(tag_name)} /> }
        },
        Route::ListPosts => {
            html! { <PostsList /> }
        },
        Route::ComposePost { id } => {
            html! { <PostCompose post_id={*id} /> }
        },
        Route::Tags => {
            html! { <TagsList /> }
        },
        Route::About => {
            html! { <About /> }
        },
        _ => {
            html! { <NotFound /> }
        },
    }
}
