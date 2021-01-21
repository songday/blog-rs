use yew::{html, Component, ComponentLink, Html, ShouldRender};

pub(crate) struct Model {}

impl Component for Model {
    type Message = ();
    type Properties = ();
    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self { Self {} }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender { false }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender { false }

    fn view(&self) -> Html {
        html! {
            <>
                <p>{"国内影视"}</p>
                <p>{"国外影视"}</p>
                <p>{"国内音乐"}</p>
                <p>{"国外音乐"}</p>
                <p>{"像素、8-bit"}</p>
                <p>{"电子游戏"}</p>
                <p>{"电子产品"}</p>
                <p>{"报刊杂志"}</p>
                <p>{"现代复古"}</p>
                <p>{"新闻旧闻"}</p>
                <p>{"生活用品：Walkman"}</p>
                <p>{"复古网站"}</p>
                <p>{"Dos、Midi、etc."}</p>
                <p>{"InternetExplorer与它的梗"}</p>
                <p>{"网页开发：曾经FrontPage还是大学的一门选修课"}</p>
                <p>{"70年代"}</p>
                <p>{"80年代"}</p>
                <p>{"90年代"}</p>
            </>
        }
    }
}
