use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::{prelude::*, Callback, Component, ComponentLink, Html, Properties};

pub struct UnlockWallet {
    link: ComponentLink<Self>,
    state: State,
    props: Props,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub on_form_submit: Callback<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    wallet_name: String,
    wallet_password: String,
    loading: bool,
}

pub enum Msg {
    UpdatePassword(String),
    UnlockWallet,
}

impl Component for UnlockWallet {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        UnlockWallet {
            props,
            link,
            state: State {
                wallet_name: "".to_string(),
                wallet_password: "".to_string(),
                loading: false,
            },
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdatePassword(value) => {
                self.state.wallet_password = value;
                false
            }
            Msg::UnlockWallet => {
                self.state.loading = true;
                let submit = self.props.on_form_submit.clone();
                let password = self.state.wallet_password.clone();
                spawn_local(async move {
                    submit.emit(password);
                });

                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        true
    }

    fn view(&self) -> Html {
        html! {
            <>
                <ybc::Subtitle>
                    { "Enter password to load wallet"  }
                </ybc::Subtitle>
                <form>
                    <ybc::Field>
                        <label class="label">{"Password"}</label>
                        <ybc::Control classes="has-icons-left">
                            <ybc::Input
                                name="password"
                                value=self.state.wallet_password.clone()
                                update=self.link.callback(|e| Msg::UpdatePassword(e))
                                r#type=ybc::InputType::Password placeholder="Password">
                            </ybc::Input>
                            <ybc::Icon classes="is-small is-left">
                                <i class="fas fa-key"></i>
                            </ybc::Icon>
                        </ybc::Control>
                    </ybc::Field>
                    <ybc::Button
                        onclick=self.link.callback(|_| Msg::UnlockWallet)
                        loading=self.state.loading
                        classes="is-primary unlock-wallet-button">{ "Unlock" }
                    </ybc::Button>
                </form>
            </>
        }
    }
}
