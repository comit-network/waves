use futures::Future;
use js_sys::Promise;
use message_types::{bs_ps, bs_ps::RpcData, cs_bs, Component};
use serde::{Deserialize, Serialize};
use wallet::WalletStatus;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_extension::browser;
use wasm_bindgen_futures::{future_to_promise, spawn_local};

// We do not support renaming the wallet for now
pub const WALLET_NAME: &str = "demo-wallet";

#[derive(Debug, Deserialize)]
struct MessageSender {
    tab: Option<Tab>,
}

#[derive(Debug, Deserialize)]
struct Tab {
    id: u32,
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    wallet::setup();

    log::info!("BS: Hello World");

    // instantiate listener to receive messages from content script or popup script
    let handle_msg = Closure::wrap(Box::new(handle_msg) as Box<dyn Fn(_, _) -> Promise>);
    browser
        .runtime()
        .on_message()
        .add_listener(handle_msg.as_ref().unchecked_ref());
    handle_msg.forget();
}

fn handle_msg(js_value: JsValue, message_sender: JsValue) -> Promise {
    if !js_value.is_object() {
        let log = format!("BS: Invalid request: {:?}", js_value);
        log::error!("{}", log);
        // TODO introduce error message
        return Promise::resolve(&JsValue::from_str("Unknown request"));
    }

    let msg: message_types::Message = match js_value.into_serde() {
        Ok(msg) => msg,
        Err(_) => {
            log::warn!("BS: Unexpected message: {:?}", js_value);
            // TODO introduce error message
            return Promise::resolve(&JsValue::from_str("Unknown request"));
        }
    };

    match (&msg.target, &msg.source) {
        (Component::Background, Component::PopUp) => {
            match js_value.into_serde() {
                Ok(msg) => handle_msg_from_ps(msg),
                Err(_) => {
                    log::warn!("BS: Unexpected message: {:?}", js_value);
                    // TODO introduce error message
                    Promise::resolve(&JsValue::from_str("Unknown request"))
                }
            }
        }
        (Component::Background, Component::Content) => {
            match (js_value.into_serde(), message_sender.into_serde()) {
                (Ok(msg), Ok(sender)) => handle_msg_from_cs(msg, sender),
                (_, _) => {
                    log::warn!("BS: Unexpected message: {:?}", js_value);
                    // TODO introduce error message
                    Promise::resolve(&JsValue::from_str("Unknown request"))
                }
            }
        }
        (_, source) => {
            log::warn!("BS: Unexpected message from {:?}", source);
            // TODO introduce error message
            Promise::resolve(&JsValue::from_str("Unknown request"))
        }
    }
}

#[macro_export]
macro_rules! map_err_from_anyhow {
    ($e:expr) => {
        match $e {
            Ok(i) => Ok(i),
            Err(e) => Err(JsValue::from_str(&format!("{:#}", e))),
        }
    };
}

async fn wallet_status(name: String) -> Result<JsValue, JsValue> {
    let status = map_err_from_anyhow!(wallet::wallet_status(name).await)?;
    log::debug!("Did not fail at line 92");

    if let WalletStatus {
        exists: false,
        loaded: false,
    } = status
    {
        return Ok(JsValue::from_serde(&bs_ps::WalletStatus::None).unwrap());
    }

    if let WalletStatus {
        exists: true,
        loaded: false,
    } = status
    {
        return Ok(JsValue::from_serde(&bs_ps::WalletStatus::NotLoaded).unwrap());
    }

    if let WalletStatus {
        exists: false,
        loaded: true,
    } = status
    {
        log::error!("unreachable: wallet cannot be loaded if it doesn't exist");
        unreachable!("wallet cannot be loaded if it doesn't exist")
    }

    let balances = map_err_from_anyhow!(wallet::get_balances(WALLET_NAME.to_string()).await)?;

    log::debug!("Balances: {:?}", &balances);

    let balances = balances
        .into_iter()
        .map(|balance| bs_ps::BalanceEntry {
            asset: balance.asset.to_string(),
            ticker: balance.ticker,
            value: balance.value,
        })
        .collect();

    let address = map_err_from_anyhow!(wallet::get_address(WALLET_NAME.to_string()).await)?;

    Ok(JsValue::from_serde(&bs_ps::WalletStatus::Loaded { balances, address }).unwrap())
}

fn handle_msg_from_ps(msg: bs_ps::Message) -> Promise {
    log::info!("BS: Received message from Popup: {:?}", msg);
    // TODO only needed to send something all the way back to PS, e.g. signed data
    let _tab_id = msg.content_tab_id;
    match msg.rpc_data {
        bs_ps::RpcData::GetWalletStatus => {
            log::debug!("Received status request from PS.");
            future_to_promise(wallet_status(WALLET_NAME.to_string()))
        }
        bs_ps::RpcData::CreateWallet(name, password) => {
            log::debug!("Creating wallet: {} ", name);
            future_to_promise(async move {
                map_err_from_anyhow!(wallet::create_new_wallet(name.clone(), password).await)?;
                wallet_status(name.to_string()).await
            })
        }
        bs_ps::RpcData::UnlockWallet(name, password) => {
            log::debug!("Received unlock request from PS");

            future_to_promise(async move {
                map_err_from_anyhow!(
                    wallet::load_existing_wallet(name.clone(), password.clone()).await
                )?;
                wallet_status(name.to_string()).await
            })
        }
        bs_ps::RpcData::Hello(data) => {
            // TODO this was just demo and should go away, for now, we keep it here
            // for testing if the whole communication chain still works
            log::error!("Currently not supported {:?}", data);
            Promise::resolve(&JsValue::from_str("UNKNOWN"))
        }
        bs_ps::RpcData::GetBalance => {
            log::debug!("Received get balance request from PS.");

            let future = async move {
                let balances =
                    map_err_from_anyhow!(wallet::get_balances(WALLET_NAME.to_string()).await)?
                        .into_iter()
                        .map(|balance| bs_ps::BalanceEntry {
                            asset: balance.asset.to_string(),
                            ticker: balance.ticker,
                            value: balance.value,
                        })
                        .collect();

                let rpc_response = bs_ps::RpcData::Balance(balances);
                let js_value = JsValue::from_serde(&rpc_response).unwrap();

                Ok(js_value)
            };
            future_to_promise(future)
        }
        RpcData::Balance(_) => {
            log::error!("Currently not supported");
            Promise::resolve(&JsValue::from_str("UNKNOWN"))
        }
    }
}

fn handle_msg_from_cs(msg: cs_bs::Message, message_sender: MessageSender) -> Promise {
    log::info!("BS: Received from CS: {:?}", &msg);

    match msg.rpc_data {
        cs_bs::RpcData::GetBalance => {
            spawn_local(async move {
                let result = wallet::get_balances(WALLET_NAME.to_string()).await;
                let tab_id = message_sender.tab.expect("tab id to exist").id;

                match result {
                    Ok(vec_balances) => {
                        //TODO export type or implement into
                        let vec_balances = vec_balances
                            .iter()
                            .map(|balance: &wallet::BalanceEntry| cs_bs::BalanceEntry {
                                asset: balance.asset.clone().to_string(),
                                ticker: balance.ticker.clone(),
                                value: balance.value,
                            })
                            .collect();

                        log::debug!("Received balance info {:?}", vec_balances);
                        let _resp = browser.tabs().send_message(
                            tab_id,
                            JsValue::from_serde(&cs_bs::Message {
                                rpc_data: cs_bs::RpcData::Balance(vec_balances),
                                target: Component::Content,
                                source: Component::Background,
                            })
                            .unwrap(),
                            JsValue::null(),
                        );
                    }
                    Err(err) => {
                        // TODO deal with error
                        log::error!("Could not get balance {:?}", err);
                    }
                }
            });
        }
        cs_bs::RpcData::GetWalletStatus => {
            spawn_local(async move {
                let result = wallet::wallet_status(WALLET_NAME.to_string()).await;
                let tab_id = message_sender.tab.expect("tab id to exist").id;

                match result {
                    Ok(wallet_status) => {
                        log::debug!("Received wallet status info {:?}", wallet_status);
                        let _resp = browser.tabs().send_message(
                            tab_id,
                            JsValue::from_serde(&cs_bs::Message {
                                rpc_data: cs_bs::RpcData::WalletStatus(wallet_status),
                                target: Component::Content,
                                source: Component::Background,
                            })
                            .unwrap(),
                            JsValue::null(),
                        );
                    }
                    Err(err) => {
                        // TODO deal with error
                        log::error!("Could not get balance {:?}", err);
                    }
                }
            });
        }
        cs_bs::RpcData::GetSellCreateSwapPayload(btc) => {
            spawn_local(async move {
                let result =
                    wallet::make_sell_create_swap_payload(WALLET_NAME.to_string(), btc).await;
                let tab_id = message_sender.tab.expect("tab id to exist").id;

                match result {
                    Ok(payload) => {
                        log::debug!("Received sell payload info {:?}", payload);
                        let _resp = browser.tabs().send_message(
                            tab_id,
                            JsValue::from_serde(&cs_bs::Message {
                                rpc_data: cs_bs::RpcData::SellCreateSwapPayload(payload),
                                target: Component::Content,
                                source: Component::Background,
                            })
                            .unwrap(),
                            JsValue::null(),
                        );
                    }
                    Err(err) => {
                        // TODO deal with error
                        log::error!("Could not get balance {:?}", err);
                    }
                }
            });
        }
        _ => {}
    }

    Promise::resolve(&JsValue::from("OK"))
}

#[derive(Serialize, Deserialize)]
struct Popup {
    pub url: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub height: u8,
    pub width: u8,
}

#[derive(Debug, Deserialize)]
struct PopupWindow {
    id: u16,
}

async fn unwrap_future<F>(future: F)
where
    F: Future<Output = Result<(), JsValue>>,
{
    if let Err(e) = future.await {
        log::error!("{:?}", &e);
    }
}

pub fn spawn<A>(future: A)
where
    A: Future<Output = Result<(), JsValue>> + 'static,
{
    spawn_local(unwrap_future(future))
}
