use serde::{Deserialize, Serialize};
use yew::{prelude::*, Callback, Component, ComponentLink, Html, Properties};

pub struct CreateWallet {
    link: ComponentLink<Self>,
    state: State,
    props: Props,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub on_form_submit: Callback<()>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    wallet_name: String,
    wallet_password: String,
}

pub enum Msg {
    UpdatePassword(String),
}

impl Component for CreateWallet {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        CreateWallet {
            props,
            link,
            state: State {
                wallet_name: "".to_string(),
                wallet_password: "".to_string(),
            },
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdatePassword(value) => {
                self.state.wallet_password = value;
            }
        }
        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        true
    }

    fn view(&self) -> Html {
        let Props { ref on_form_submit } = self.props;
        let onclick = on_form_submit.reform(move |_| ());

        html! {
            <>
                <p>{"Wallet does not exist"}</p>
                <form>
                    <input
                       placeholder="Name"
                       value=&self.state.wallet_name
                       disabled=true
                       />
                    <input
                       placeholder="Password"
                       value=&self.state.wallet_password
                       oninput=self.link.callback(|e: InputData| Msg::UpdatePassword(e.value))
                       />
                    <button onclick=onclick>{ "Create" }</button>
                </form>
            </>
        }
    }
}
