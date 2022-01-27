use blog_frontend::app::App;
use yew::{html, prelude::*, Html};
use yew_router::prelude::*;

#[function_component(Main)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <App />
        </BrowserRouter>
    }
}

fn main() {
    yew::start_app::<Main>();
}
