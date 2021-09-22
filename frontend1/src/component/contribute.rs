use yew::{html, Component, Context, Html};

pub(crate) struct Model {}

impl Component for Model {
    type Message = ();
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self { Self {} }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> ShouldRender { false }

    fn changed(&mut self, _ctx: &Context<Self>) -> ShouldRender { false }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <h2>{"欢迎投稿"}</h2>
                <p>{"请发邮件到：80@songday.com"}</p>
            </>
        }
    }
}
