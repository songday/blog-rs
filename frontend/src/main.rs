use yew::prelude::*;
use yew_router::prelude::*;

use blog_frontend::router::{Route, switch};

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
            <main>
                <Router<Route> render={Router::render(switch)} />
            </main>
            <footer class="footer">
                <div class="content has-text-centered">
                    { "Powered by " }
                    <a href="https://yew.rs">{ "Yew" }</a>
                    { " using " }
                    <a href="https://bulma.io">{ "Bulma" }</a>
                    { " and images from " }
                    <a href="https://unsplash.com">{ "Unsplash" }</a>
                </div>
            </footer>
        </>
    }
}

fn main() {
    yew::start_app::<App>();
}