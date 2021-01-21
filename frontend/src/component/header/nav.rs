use yew::{prelude::*, virtual_dom::VNode};
use yew_router::prelude::*;

use crate::app::AppRoute;

pub(crate) struct Model {}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self { Self {} }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender { false }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender { false }

    fn view(&self) -> VNode {
        html! {
            <>
                <div><RouterAnchor<AppRoute> route=AppRoute::About> {"关于我们"} </RouterAnchor<AppRoute>></div>
                // <RouterButton<AppRoute> route=AppRoute::About> {"About"} </RouterButton<AppRoute>>
            </>
        }
    }
}
