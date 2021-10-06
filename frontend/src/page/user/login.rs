use yew::prelude::*;
use yew_router::prelude::*;

pub struct UserLogin;

impl Component for UserLogin {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <div class="field">
                  <label class="label">Username</label>
                  <div class="control">
                    <input class="input" type="text" placeholder="Username">
                  </div>
                </div>

                <div class="field">
                  <label class="label">Password</label>
                  <div class="control">
                    <input class="input" type="password" placeholder="Password">
                  </div>
                </div>

                <button class="button is-link is-light">Sign In</button>
            </>
        }
    }
}