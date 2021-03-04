use bs_ps::TransactionData;
use conquer_once::Lazy;
use futures::{lock::Mutex, Future};
use js_sys::Promise;
use message_types::{bs_ps, cs_bs, Component};
use serde::{Deserialize, Serialize};
use wallet::WalletStatus;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_extension::browser;
use wasm_bindgen_futures::{future_to_promise, spawn_local};

// We do not support renaming the wallet for now
pub const WALLET_NAME: &str = "demo-wallet";

static SIGN_TX: Lazy<Mutex<Option<TransactionData>>> = Lazy::new(Mutex::default);

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

async fn background_status(
    name: String,
    sign_tx: Option<TransactionData>,
) -> Result<JsValue, JsValue> {
    let status = map_err_from_anyhow!(wallet::wallet_status(name).await)?;
    if let WalletStatus {
        exists: false,
        loaded: false,
    } = status
    {
        log::debug!("Wallet does not exist");
        return Ok(JsValue::from_serde(&bs_ps::BackgroundStatus::new(
            bs_ps::WalletStatus::None,
            sign_tx,
        ))
        .unwrap());
    }

    if let WalletStatus {
        exists: true,
        loaded: false,
    } = status
    {
        log::debug!("Wallet exists but not loaded");
        return Ok(JsValue::from_serde(&bs_ps::BackgroundStatus::new(
            bs_ps::WalletStatus::NotLoaded,
            sign_tx,
        ))
        .unwrap());
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
    let balances = balances
        .into_iter()
        .map(|balance| bs_ps::BalanceEntry {
            asset: balance.asset.to_string(),
            ticker: balance.ticker,
            value: balance.value,
        })
        .collect();

    let address = map_err_from_anyhow!(wallet::get_address(WALLET_NAME.to_string()).await)?;

    Ok(JsValue::from_serde(&bs_ps::BackgroundStatus::new(
        bs_ps::WalletStatus::Loaded {
            balances,
            address: address.to_string(),
        },
        sign_tx,
    ))
    .unwrap())
}

fn handle_msg_from_ps(msg: bs_ps::Message) -> Promise {
    log::info!("BS: Received message from Popup: {:?}", msg);
    match msg.rpc_data {
        bs_ps::RpcData::GetWalletStatus => {
            log::debug!("Received status request from PS.");
            future_to_promise(async {
                let sign_tx = SIGN_TX.lock().await.clone();
                background_status(WALLET_NAME.to_string(), sign_tx).await
            })
        }
        bs_ps::RpcData::CreateWallet(name, password) => {
            log::debug!("Creating wallet: {} ", name);
            future_to_promise(async move {
                map_err_from_anyhow!(wallet::create_new_wallet(name.clone(), password).await)?;
                let sign_tx = SIGN_TX.lock().await.clone();
                background_status(name.to_string(), sign_tx).await
            })
        }
        bs_ps::RpcData::UnlockWallet(name, password) => {
            log::debug!("Received unlock request from PS");

            future_to_promise(async move {
                map_err_from_anyhow!(
                    wallet::load_existing_wallet(name.clone(), password.clone()).await
                )?;
                let sign_tx = SIGN_TX.lock().await.clone();
                background_status(name.to_string(), sign_tx).await
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
        bs_ps::RpcData::SignAndSend { tx_hex, tab_id } => {
            future_to_promise(async move {
                let result =
                    wallet::sign_and_send_swap_transaction(WALLET_NAME.to_string(), tx_hex.clone())
                        .await;

                match result {
                    Ok(txid) => {
                        log::debug!("Received swap txid info {:?}", txid);
                        let _resp = browser.tabs().send_message(
                            tab_id,
                            JsValue::from_serde(&cs_bs::Message {
                                rpc_data: cs_bs::RpcData::SwapTxid(txid),
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

                let mut guard = SIGN_TX.lock().await;
                if let Some(TransactionData { hex, .. }) = &*guard {
                    if hex == &tx_hex {
                        let _ = guard.take();
                    }
                }

                let sign_tx = guard.clone();
                background_status(WALLET_NAME.to_string(), sign_tx).await
            })
        }
        bs_ps::RpcData::Balance(_) => {
            log::error!("Currently not supported");
            Promise::resolve(&JsValue::from_str("UNKNOWN"))
        }
    }
}

fn handle_msg_from_cs(msg: cs_bs::Message, message_sender: MessageSender) -> Promise {
    log::info!("BS: Received from CS: {:?}", &msg);

    match msg.rpc_data {
        cs_bs::RpcData::GetWalletStatus => {
            spawn_local(async move {
                let result = wallet::wallet_status(WALLET_NAME.to_string()).await;
                let tab_id = message_sender.tab.expect("tab id to exist").id;

                match result {
                    Ok(background_status) => {
                        log::debug!("Received wallet status info {:?}", background_status);
                        let _resp = browser.tabs().send_message(
                            tab_id,
                            JsValue::from_serde(&cs_bs::Message {
                                rpc_data: cs_bs::RpcData::WalletStatus(background_status),
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
        cs_bs::RpcData::GetBuyCreateSwapPayload(usdt) => {
            spawn_local(async move {
                let result =
                    wallet::make_buy_create_swap_payload(WALLET_NAME.to_string(), usdt).await;
                let tab_id = message_sender.tab.expect("tab id to exist").id;

                match result {
                    Ok(payload) => {
                        log::debug!("Received buy payload info {:?}", payload);
                        let _resp = browser.tabs().send_message(
                            tab_id,
                            JsValue::from_serde(&cs_bs::Message {
                                rpc_data: cs_bs::RpcData::BuyCreateSwapPayload(payload),
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
        cs_bs::RpcData::SignAndSend(tx_hex) => spawn_local(async move {
            let result = wallet::extract_trade(WALLET_NAME.to_string(), tx_hex.clone()).await;
            let tab_id = message_sender.tab.expect("tab id to exist").id;

            match result {
                Ok(trade) => {
                    log::debug!("Received trade info {:?}", trade);

                    let tx_data = TransactionData {
                        hex: tx_hex,
                        decoded: trade,
                        tab_id,
                    };

                    let mut guard = SIGN_TX.lock().await;
                    guard.replace(tx_data);
                }
                Err(err) => {
                    // TODO deal with error
                    log::error!("Could not get trade info {:?}", err);
                }
            }
        }),
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
