use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    pub post_id: Option<u64>,
}

pub struct PostCompose {
    post_id: Option<u64>,
}

impl Component for PostCompose {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            post_id: None,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        if self.post_id.is_none() && ctx.props().post_id.is_none() {
            return false;
        }
        if self.post_id.is_some() && ctx.props().post_id.is_some() {
            return self.post_id.unwrap() != ctx.props().post_id.unwrap();
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let Self { post_id } = self;

        html! {
            <>
            </>
        }
    }
}