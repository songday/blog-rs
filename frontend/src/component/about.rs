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
                <h2>{"欢迎来到这里"}</h2>
                <p>{"聚焦 70、80、90年代，怀旧、复古、像素化的东西"}</p>
                <p>{"本网站使用微软的 ASP 技术构建"}</p>
                <p>{"开发工具是：DreamWeaver UltraDev"}</p>
                <p>{"开发环境是：Windows 98 PWS"}</p>
                <p>{"网站托管主机所使用的系统是：Windows 2000 Server SP4 版本"}</p>
            </>
        }
    }
}
