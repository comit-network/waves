use anyhow::Context;
use conquer_once::Lazy;
use futures::lock::Mutex;
use js_sys::Promise;
use message_types::{
    bs_ps::{self, BackgroundStatus, TransactionData, WalletStatus},
    SignError, ToBackground, ToPage,
};
use serde::Deserialize;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_extension::{browser, object};
use wasm_bindgen_futures::{future_to_promise, spawn_local};

pub const WALLET_NAME: &str = "demo-wallet";

static SIGN_TX: Lazy<Mutex<Option<TransactionData>>> = Lazy::new(Mutex::default);
static BADGE_COUNTER: Lazy<Mutex<Option<u32>>> = Lazy::new(|| Mutex::new(Some(0)));

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

fn handle_msg(msg: JsValue, sender: JsValue) -> Promise {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Msg {
        FromContent(ToBackground),
        FromPopup(bs_ps::ToBackground),
    }

    let sender = sender.into_serde::<MessageSender>().unwrap();

    match msg.into_serde() {
        Ok(Msg::FromPopup(msg)) => handle_msg_from_ps(msg),
        Ok(Msg::FromContent(msg)) => handle_msg_from_cs(msg, sender),
        _ => {
            log::warn!("BS: Unexpected message: {:?}", msg);
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

async fn wallet_status(name: String, sign_tx: Option<TransactionData>) -> Result<JsValue, JsValue> {
    let status = map_err_from_anyhow!(wallet::wallet_status(name).await)?;
    if let wallet::WalletStatus {
        exists: false,
        loaded: false,
    } = status
    {
        log::debug!("Wallet does not exist");
        return Ok(
            JsValue::from_serde(&BackgroundStatus::new(WalletStatus::None, sign_tx)).unwrap_throw(),
        );
    }

    if let wallet::WalletStatus {
        exists: true,
        loaded: false,
    } = status
    {
        log::debug!("Wallet exists but not loaded");
        return Ok(
            JsValue::from_serde(&BackgroundStatus::new(WalletStatus::NotLoaded, sign_tx))
                .unwrap_throw(),
        );
    }

    if let wallet::WalletStatus {
        exists: false,
        loaded: true,
    } = status
    {
        log::error!("unreachable: wallet cannot be loaded if it doesn't exist");
        unreachable!("wallet cannot be loaded if it doesn't exist")
    }

    let address = map_err_from_anyhow!(wallet::get_address(WALLET_NAME.to_string())
        .await
        .context("could not get address"))?;

    Ok(JsValue::from_serde(&BackgroundStatus::new(
        WalletStatus::Loaded {
            address: address.to_string(),
        },
        sign_tx,
    ))
    .unwrap_throw())
}

fn handle_msg_from_ps(msg: bs_ps::ToBackground) -> Promise {
    log::info!("BS: Received message from PS: {:?}", msg);

    match msg {
        bs_ps::ToBackground::BackgroundStatusRequest => future_to_promise(async {
            let sign_tx = SIGN_TX.lock().await.clone();
            wallet_status(WALLET_NAME.to_string(), sign_tx).await
        }),
        bs_ps::ToBackground::CreateWalletRequest(name, password) => future_to_promise(async move {
            map_err_from_anyhow!(wallet::create_new_wallet(name.clone(), password).await)?;
            let sign_tx = SIGN_TX.lock().await.clone();
            wallet_status(name.to_string(), sign_tx).await
        }),
        bs_ps::ToBackground::UnlockRequest(name, password) => future_to_promise(async move {
            map_err_from_anyhow!(
                wallet::load_existing_wallet(name.clone(), password.clone()).await
            )?;
            let sign_tx = SIGN_TX.lock().await.clone();
            wallet_status(name.to_string(), sign_tx).await
        }),
        bs_ps::ToBackground::BalanceRequest => future_to_promise(async move {
            let balances =
                map_err_from_anyhow!(wallet::get_balances(WALLET_NAME.to_string()).await)?;
            let rpc_response = bs_ps::ToPopup::BalanceResponse(balances);
            let js_value = JsValue::from_serde(&rpc_response).unwrap();

            Ok(js_value)
        }),
        bs_ps::ToBackground::SignRequest { tx_hex, tab_id } => {
            future_to_promise(async move {
                let result =
                    wallet::sign_and_send_swap_transaction(WALLET_NAME.to_string(), tx_hex.clone())
                        .await;

                match result {
                    Ok(txid) => {
                        log::debug!("Received swap txid info {:?}", txid);
                        let _resp = browser.tabs().send_message(
                            tab_id,
                            JsValue::from_serde(&ToPage::SignResponse(Ok(txid))).unwrap(),
                            JsValue::null(),
                        );
                    }
                    Err(err) => {
                        // TODO deal with error
                        log::error!("Could not sign and send swap transaction {:?}", err);
                    }
                }

                // remove TX from the guard after signing
                let mut guard = SIGN_TX.lock().await;
                if let Some(TransactionData { hex, .. }) = &*guard {
                    if hex == &tx_hex {
                        let _ = guard.take();
                    }
                }

                // reset badge counter
                decrement_badge_counter().await;

                let sign_tx = guard.clone();
                wallet_status(WALLET_NAME.to_string(), sign_tx).await
            })
        }

        bs_ps::ToBackground::Reject { tx_hex, tab_id } => {
            future_to_promise(async move {
                // remove TX from the guard
                let mut guard = SIGN_TX.lock().await;
                if let Some(TransactionData { hex, .. }) = &*guard {
                    if hex == &tx_hex {
                        let _ = guard.take();
                    }
                }

                // reset badge counter
                log::debug!("Decrementing badge counter");
                decrement_badge_counter().await;
                log::debug!("Decrementing badge counter: done");

                log::debug!("Rejected swap");
                let _resp = browser.tabs().send_message(
                    tab_id,
                    JsValue::from_serde(&ToPage::SignResponse(Err(SignError::Rejected))).unwrap(),
                    JsValue::null(),
                );

                let sign_tx = guard.clone();
                wallet_status(WALLET_NAME.to_string(), sign_tx).await
            })
        }
    }
}

fn handle_msg_from_cs(msg: ToBackground, message_sender: MessageSender) -> Promise {
    log::info!("BS: Received from CS: {:?}", &msg);

    match msg {
        ToBackground::StatusRequest => {
            spawn_local(async move {
                let result = wallet::wallet_status(WALLET_NAME.to_string()).await;
                let tab_id = message_sender.tab.expect("tab id to exist").id;

                match result {
                    Ok(payload) => {
                        log::debug!("Received wallet status info {:?}", payload);
                        let _resp = browser.tabs().send_message(
                            tab_id,
                            JsValue::from_serde(&ToPage::StatusResponse(payload)).unwrap(),
                            JsValue::null(),
                        );
                    }
                    Err(err) => {
                        // TODO deal with error
                        log::error!("Could not get wallet status {:?}", err);
                    }
                }
            });
        }
        ToBackground::SellRequest(btc) => {
            spawn_local(async move {
                let result =
                    wallet::make_sell_create_swap_payload(WALLET_NAME.to_string(), btc).await;
                let tab_id = message_sender.tab.expect("tab id to exist").id;

                match result {
                    Ok(payload) => {
                        log::debug!("Received sell payload info {:?}", payload);
                        let _resp = browser.tabs().send_message(
                            tab_id,
                            JsValue::from_serde(&ToPage::SellResponse(payload)).unwrap(),
                            JsValue::null(),
                        );
                    }
                    Err(err) => {
                        // TODO deal with error
                        log::error!("Could not get sell create swap payload {:?}", err);
                    }
                }
            });
        }
        ToBackground::BuyRequest(usdt) => {
            spawn_local(async move {
                let result =
                    wallet::make_buy_create_swap_payload(WALLET_NAME.to_string(), usdt).await;
                let tab_id = message_sender.tab.expect("tab id to exist").id;

                match result {
                    Ok(payload) => {
                        log::debug!("Received buy payload info {:?}", payload);
                        let _resp = browser.tabs().send_message(
                            tab_id,
                            JsValue::from_serde(&ToPage::BuyResponse(payload)).unwrap(),
                            JsValue::null(),
                        );
                    }
                    Err(err) => {
                        // TODO deal with error
                        log::error!("Could not get buy create swap payload {:?}", err);
                    }
                }
            });
        }
        ToBackground::SignRequest(tx_hex) => spawn_local(async move {
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

                    // increment badge counter
                    increment_badge_counter().await;
                }
                Err(err) => {
                    // TODO deal with error
                    log::error!("Could not get trade info {:?}", err);
                }
            }
        }),
    }

    Promise::resolve(&JsValue::from("OK"))
}

async fn decrement_badge_counter() {
    let mut guard = BADGE_COUNTER.lock().await;
    let counter = guard.clone().unwrap_or(0);
    let new_counter = if counter > 0 { counter - 1 } else { 0 };
    guard.replace(new_counter);
    set_badge(new_counter);
}

async fn increment_badge_counter() {
    let mut guard = BADGE_COUNTER.lock().await;
    let counter = guard.clone().unwrap_or(0);
    let new_counter = counter + 1;
    guard.replace(new_counter);
    set_badge(new_counter);
}

fn set_badge(counter: u32) {
    let counter = if counter == 0 {
        "".to_string()
    } else {
        counter.to_string()
    };
    browser.browser_action().set_badge_text(&object! {
        "text": counter,
    });
    browser
        .browser_action()
        .set_badge_background_color(&object! {
            // green
            "color": "#03D607",
        });
}
