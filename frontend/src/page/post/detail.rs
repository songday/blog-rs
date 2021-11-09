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

    fn create(_ctx: &Context<Self>) -> Self {
        Self { post_id: 0 }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.post_id != ctx.props().post_id
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let Self { post_id } = self;

        html! {
            <>
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
                <div class="section container">
                    <article class="media block box my-6">
                        <div class="media-content">
                            <div class="content">
                                <p class="is-family-secondary">
                                    {"{ &quote.content }"}
                                </p>
                            </div>
                        </div>
                    </article>
                </div>
            </>
        }
    }
}
