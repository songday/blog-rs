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
                <h2>{"欢迎投稿"}</h2>
                <p>{"请发邮件到：80@songday.com"}</p>
            </>
        }
    }
}
