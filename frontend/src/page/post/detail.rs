use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    pub post_id: u64,
}

pub struct PostDetail {
    post_id: u64,
}

impl Component for PostDetail {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            post_id: 0,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.post_id != ctx.props().post_id
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let Self { post_id } = self;

        html! {
            <main>
                <section class="hero is-medium is-light has-background">
                  <img src="" class="hero-background is-transparent"/>
                  <div class="hero-body">
                    <div class="container">
                    <p class="title">
                      {"Medium hero"}
                    </p>
                    <p class="subtitle">
                      {"Medium subtitle"}
                    </p>
                    <div class="tags">
                        <span class="tag is-info">{"tag"}</span>
                    </div>
                    </div>
                  </div>
                </section>
            </main>
        }
    }
}