use alloc::vec::Vec;
use core::convert::Into;

use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use crate::util::Error;

pub(crate) struct ShowErrors {
    props: Props,
}

#[derive(Properties, Clone)]
pub(crate) struct Props {
    pub error: Option<Error>,
}

impl Component for ShowErrors {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self { ShowErrors { props } }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender { true }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        if let Some(error) = &self.props.error {
            html! {
                <ul class="error-messages">
                    {
                        match error {
                            Error::BusinessError(msg) => {
                                html! {
                                    <li>{msg}</li>
                                }
                            },
                            _ => {
                                html! {
                                    <li>{error}</li>
                                }
                            }

                        }
                    }
                </ul>
            }
        } else {
            html! {}
        }
    }
}
