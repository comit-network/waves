use crate::components::{
    create_wallet_form::CreateWallet, unlock_wallet_form::UnlockWallet,
    wallet_details::WalletDetails,
};
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
    CreateWallet,
    UnlockWallet,
    WalletStatus(WalletStatus),
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
        return match msg {
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
                false
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
                false
            }
            Msg::WalletStatus(wallet_status) => {
                self.state.wallet_status = wallet_status;
                true
            }
        };
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        true
    }

    fn view(&self) -> Html {
        let wallet_form = match &self.state.wallet_status {
            WalletStatus::Loaded { address, balances } => {
                html! {
                    <>
                        <WalletDetails address=address balances=balances></WalletDetails>
                    </>
                }
            }
            WalletStatus::NotLoaded => {
                html! {
                    <>
                        <UnlockWallet on_form_submit=self.link.callback(|_| Msg::UnlockWallet)></UnlockWallet>
                    </>
                }
            }
            WalletStatus::None => {
                html! {
                    <>
                        <CreateWallet on_form_submit=self.link.callback(|_| Msg::CreateWallet)></CreateWallet>
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
