use js_sys::Promise;
use message_types::{bs_ps, Component as MessageComponent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen_extension::browser;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use yew::prelude::*;

// We do not support renaming the wallet for now
pub const WALLET_NAME: &str = "demo-wallet";

pub struct App {
    link: ComponentLink<Self>,
    content_tab_id: u32,
    state: State,
}

pub enum Msg {
    UpdatePassword(String),
    CreateWallet,
    UnlockWallet,
    WalletStatus(WalletStatus),
    BalanceUpdate(Vec<bs_ps::BalanceEntry>),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    wallet_name: String,
    wallet_password: String,
    wallet_status: WalletStatus,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum WalletStatus {
    None,
    NotLoaded,
    Loaded {
        balances: Vec<bs_ps::BalanceEntry>,
        address: String,
    },
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        log::debug!("PopupApp: creating...");
        let window = web_sys::window().expect("no global `window` exists");

        let inner_link = link.clone();
        let msg = bs_ps::Message {
            rpc_data: bs_ps::RpcData::GetWalletStatus,
            target: MessageComponent::Background,
            source: MessageComponent::PopUp,
            content_tab_id: 0,
        };
        send_to_backend(
            msg,
            Box::new(move |response| {
                log::debug!("Wallet status after creating: {:?}", response);

                if let Ok(response) = response {
                    if let Ok(msg) = response.into_serde() {
                        inner_link.send_message(Msg::WalletStatus(msg));
                    }
                }
            }),
        );

        // TODO this will go away in one way or the other but for now is needed for the demo message
        let url = Url::parse(&window.location().href().unwrap()).unwrap();
        let queries: HashMap<String, String> = url.query_pairs().into_owned().collect();
        let content_tab_id = if let Some(tab_id) = queries.get("content_tab_id") {
            tab_id.parse::<u32>().expect("Tab id should be a number")
        } else {
            1
        };
        log::debug!("Content tab ID = {}", content_tab_id);

        App {
            link,
            content_tab_id,
            state: State {
                wallet_name: WALLET_NAME.to_string(),
                wallet_password: "".to_string(),
                wallet_status: WalletStatus::None,
            },
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdatePassword(value) => {
                self.state.wallet_password = value;
            }
            Msg::UnlockWallet => {
                let inner_link = self.link.clone();
                let msg = bs_ps::Message {
                    rpc_data: bs_ps::RpcData::UnlockWallet(
                        self.state.wallet_name.clone(),
                        self.state.wallet_password.clone(),
                    ),
                    target: message_types::Component::Background,
                    source: message_types::Component::PopUp,
                    content_tab_id: 0,
                };
                send_to_backend(
                    msg,
                    Box::new(move |response| {
                        if let Ok(response) = response {
                            if let Ok(wallet_status) = response.into_serde() {
                                inner_link.send_message(Msg::WalletStatus(wallet_status));
                            }
                        }
                    }),
                );
                return false;
            }
            Msg::CreateWallet => {
                let inner_link = self.link.clone();
                let msg = bs_ps::Message {
                    rpc_data: bs_ps::RpcData::CreateWallet(
                        self.state.wallet_name.clone(),
                        self.state.wallet_password.clone(),
                    ),
                    target: message_types::Component::Background,
                    source: message_types::Component::PopUp,
                    content_tab_id: 0,
                };
                send_to_backend(
                    msg,
                    Box::new(move |response| {
                        if response.is_ok() {
                            inner_link.send_message(Msg::WalletStatus(WalletStatus::NotLoaded));
                        }
                    }),
                );
                return false;
            }
            Msg::WalletStatus(wallet_status) => {
                self.state.wallet_status = wallet_status;
                return true;
            }
            Msg::BalanceUpdate(wallet_balances) => match &self.state.wallet_status {
                WalletStatus::None => return false,
                WalletStatus::NotLoaded => return true,
                WalletStatus::Loaded { address, .. } => {
                    self.state.wallet_status = WalletStatus::Loaded {
                        address: address.clone(),
                        balances: wallet_balances,
                    };
                    return true;
                }
            },
        }
        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        true
    }

    fn view(&self) -> Html {
        let render_item = |balance: &bs_ps::BalanceEntry| -> Html {
            html! {
            <li> {balance.asset.clone()} </li>
            }
        };
        let wallet_form = match &self.state.wallet_status {
            WalletStatus::Loaded { address, balances } => {
                html! {
                    <>
                        <p>{"Wallet exists"}</p>
                        <p>{format!("Address: {}", address)}</p>
                        <p>{"Balances:"}</p>
                        <ul class="item-list">
                    { balances.iter().map(render_item).collect::<Html>() }
                    </ul>
                    </>
                }
            }
            WalletStatus::NotLoaded => {
                html! {
                    <>
                        <p>{"Wallet exists but not loaded"}</p>
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
                            <button onclick=self.link.callback(|_| Msg::UnlockWallet)>{ "Unlock" }</button>
                        </form>
                    </>
                }
            }
            WalletStatus::None => {
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
                            <button onclick=self.link.callback(|_| Msg::CreateWallet)>{ "Create" }</button>
                        </form>
                    </>
                }
            }
        };

        let faucet_button = match &self.state.wallet_status {
            WalletStatus::Loaded { address, .. } => {
                let address = address.clone();
                html! {
                    <>
                        <button onclick=self.link.batch_callback(
                            move |_| {
                                faucet(address.to_string());
                                vec![]
                            })>{ "Faucet" }</button>
                    </>
                }
            }
            _ => html! {},
        };

        html! {
            <div>
                <p>{ "Waves Wallet" }</p>
                { wallet_form }
                // TODO: Feature flag this
                {faucet_button}
            </div>
        }
    }

    fn rendered(&mut self, _first_render: bool) {}

    fn destroy(&mut self) {}
}

fn faucet(address: String) {
    spawn_local(async move {
        let client = reqwest::Client::new();
        match client
            .post(format!("http://127.0.0.1:3030/api/faucet/{}", address).as_str())
            .send()
            .await
        {
            Ok(_) => {}
            Err(e) => log::error!("Call to faucet failed: {:?}", e),
        };
    })
}

fn send_to_backend(message: bs_ps::Message, callback: Box<dyn Fn(Result<JsValue, JsValue>)>) {
    spawn_local(async move {
        let js_value = JsValue::from_serde(&message).unwrap();
        let promise: Promise = browser.runtime().send_message(None, &js_value, None);
        let result = JsFuture::from(promise).await;
        callback(result)
    });
}
