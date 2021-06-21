use anyhow::{bail, Context, Result};
use conquer_once::Lazy;
use elements::encode::serialize_hex;
use futures::lock::Mutex;
use js_sys::Promise;
use message_types::{
    bs_ps::{self, BackgroundStatus, LoanData, SignState, TransactionData, WalletStatus},
    ips_bs::{self, SignAndSendError, SignLoanError},
};
use serde::Deserialize;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_extension::{browser, object};
use wasm_bindgen_futures::{future_to_promise, spawn_local};

pub const WALLET_NAME: &str = "demo-wallet";

static SIGN_STATE: Lazy<Mutex<SignState>> = Lazy::new(Mutex::default);
static BADGE_COUNTER: Lazy<Mutex<Option<u32>>> = Lazy::new(|| Mutex::new(Some(0)));

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    wallet::setup();

    log::info!("BS: Hello World");

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
        FromContent(ips_bs::ToBackground),
        FromPopup(bs_ps::ToBackground),
    }

    let sender = sender.into_serde::<MessageSender>().unwrap();

    match msg.into_serde() {
        Ok(Msg::FromPopup(msg)) => handle_msg_from_ps(msg),
        Ok(Msg::FromContent(msg)) => handle_msg_from_cs(msg, sender),
        _ => {
            log::warn!("BS: Unexpected message: {:?}", msg);
            Promise::resolve(&JsValue::from_str("Unknown request"))
        }
    }
}

fn handle_msg_from_ps(msg: bs_ps::ToBackground) -> Promise {
    log::info!("BS: Received message from PS: {:?}", msg);

    match msg {
        bs_ps::ToBackground::BackgroundStatusRequest => future_to_promise(async {
            let sign_state = SIGN_STATE.lock().await.clone();
            let status =
                map_err_from_anyhow!(background_status(WALLET_NAME.to_string(), sign_state).await)?;
            let msg = JsValue::from_serde(&status).unwrap();

            Ok(msg)
        }),
        bs_ps::ToBackground::CreateWalletRequest(name, password) => future_to_promise(async move {
            map_err_from_anyhow!(wallet::create_new_wallet(name.clone(), password).await)?;

            let sign_state = SIGN_STATE.lock().await.clone();
            let status =
                map_err_from_anyhow!(background_status(WALLET_NAME.to_string(), sign_state).await)?;
            let msg = JsValue::from_serde(&status).unwrap();

            Ok(msg)
        }),
        bs_ps::ToBackground::UnlockRequest(name, password) => future_to_promise(async move {
            map_err_from_anyhow!(
                wallet::load_existing_wallet(name.clone(), password.clone()).await
            )?;

            let sign_state = SIGN_STATE.lock().await.clone();
            let status =
                map_err_from_anyhow!(background_status(WALLET_NAME.to_string(), sign_state).await)?;
            let msg = JsValue::from_serde(&status).unwrap();

            Ok(msg)
        }),
        bs_ps::ToBackground::BalanceRequest => future_to_promise(async move {
            let balances =
                map_err_from_anyhow!(wallet::get_balances(WALLET_NAME.to_string()).await)?;
            let msg = bs_ps::ToPopup::BalanceResponse(balances);
            let msg = JsValue::from_serde(&msg).unwrap();

            Ok(msg)
        }),
        bs_ps::ToBackground::SignRequest { tx_hex, tab_id } => {
            future_to_promise(async move {
                let res =
                    wallet::sign_and_send_swap_transaction(WALLET_NAME.to_string(), tx_hex.clone())
                        .await
                        .map_err(ips_bs::SignAndSendError::from);

                let _resp = browser.tabs().send_message(
                    tab_id,
                    JsValue::from_serde(&ips_bs::ToPage::SignResponse(res)).unwrap(),
                    JsValue::null(),
                );

                // remove TX from the guard after signing
                let mut guard = SIGN_STATE.lock().await;
                if let SignState::Trade(TransactionData { hex, .. }) = &*guard {
                    if hex == &tx_hex {
                        guard.unset();
                    }
                }

                decrement_badge_counter().await;

                // TODO: We should send back a specific message to the
                // pop-up after attempting to sign and send the
                // transaction
                let sign_state = guard.clone();
                let status = map_err_from_anyhow!(
                    background_status(WALLET_NAME.to_string(), sign_state).await
                )?;
                let msg = JsValue::from_serde(&status).unwrap();

                Ok(msg)
            })
        }
        bs_ps::ToBackground::Reject { tx_hex, tab_id } => {
            future_to_promise(async move {
                // TODO: Extract into helper function
                // remove TX from the guard
                let mut guard = SIGN_STATE.lock().await;
                if let SignState::Trade(TransactionData { hex, .. }) = &*guard {
                    if hex == &tx_hex {
                        guard.unset();
                    }
                }

                decrement_badge_counter().await;

                log::debug!("Rejected swap transaction {}", tx_hex);
                let _resp = browser.tabs().send_message(
                    tab_id,
                    JsValue::from_serde(&ips_bs::ToPage::SignResponse(Err(
                        SignAndSendError::Rejected,
                    )))
                    .unwrap(),
                    JsValue::null(),
                );

                let sign_state = guard.clone();
                let status = map_err_from_anyhow!(
                    background_status(WALLET_NAME.to_string(), sign_state).await
                )?;
                let msg = JsValue::from_serde(&status).unwrap();

                Ok(msg)
            })
        }
        bs_ps::ToBackground::SignLoan { details, tab_id } => {
            future_to_promise(async move {
                let res = wallet::sign_loan(WALLET_NAME.to_string())
                    .await
                    .map(|tx| serialize_hex(&tx))
                    .map_err(ips_bs::SignLoanError::from);

                let _resp = browser.tabs().send_message(
                    tab_id,
                    JsValue::from_serde(&ips_bs::ToPage::LoanTransaction(res)).unwrap(),
                    JsValue::null(),
                );

                // remove TX from the guard after signing
                let mut guard = SIGN_STATE.lock().await;
                if let SignState::Loan(LoanData {
                    details: state_details,
                    ..
                }) = &*guard
                {
                    if details == *state_details {
                        guard.unset();
                    }
                }

                decrement_badge_counter().await;
                let sign_state = guard.clone();
                let status = map_err_from_anyhow!(
                    background_status(WALLET_NAME.to_string(), sign_state).await
                )?;
                let msg = JsValue::from_serde(&status).unwrap();

                Ok(msg)
            })
        }
        bs_ps::ToBackground::RejectLoan { details, tab_id } => future_to_promise(async move {
            let mut guard = SIGN_STATE.lock().await;
            if let SignState::Loan(LoanData {
                details: state_details,
                ..
            }) = &*guard
            {
                if details == *state_details {
                    guard.unset();
                }
            }

            decrement_badge_counter().await;

            log::debug!("Rejected signing loan {:?}", details);
            let _resp = browser.tabs().send_message(
                tab_id,
                JsValue::from_serde(&ips_bs::ToPage::LoanTransaction(Err(
                    SignLoanError::Rejected,
                )))
                .unwrap(),
                JsValue::null(),
            );

            let sign_state = guard.clone();
            let status =
                map_err_from_anyhow!(background_status(WALLET_NAME.to_string(), sign_state).await)?;
            let msg = JsValue::from_serde(&status).unwrap();

            Ok(msg)
        }),
        bs_ps::ToBackground::WithdrawAll(address) => future_to_promise(async move {
            let txid = map_err_from_anyhow!(
                wallet::withdraw_everything_to(WALLET_NAME.to_string(), address).await
            )?;
            let js_value = JsValue::from_serde(&txid).unwrap();

            Ok(js_value)
        }),
        bs_ps::ToBackground::RepayLoan(txid) => future_to_promise(async move {
            let _txid = map_err_from_anyhow!(
                wallet::repay_loan(WALLET_NAME.to_string(), txid.to_string()).await
            )?;
            let sign_state = SIGN_STATE.lock().await;
            let status = map_err_from_anyhow!(
                background_status(WALLET_NAME.to_string(), sign_state.clone()).await
            )?;
            let msg = JsValue::from_serde(&status).unwrap();

            Ok(msg)
        }),
    }
}

fn handle_msg_from_cs(msg: ips_bs::ToBackground, message_sender: MessageSender) -> Promise {
    log::info!("BS: Received from CS: {:?}", &msg);

    match msg {
        ips_bs::ToBackground::WalletStatusRequest => {
            spawn_local(async move {
                let tab_id = message_sender.tab.expect("tab id to exist").id;
                let res = wallet::wallet_status(WALLET_NAME.to_string())
                    .await
                    .map_err(|e| ips_bs::StatusError(format!("{:#}", e)));

                let _resp = browser.tabs().send_message(
                    tab_id,
                    JsValue::from_serde(&ips_bs::ToPage::StatusResponse(res)).unwrap(),
                    JsValue::null(),
                );
            });
        }
        ips_bs::ToBackground::SellRequest(btc) => {
            spawn_local(async move {
                let tab_id = message_sender.tab.expect("tab id to exist").id;
                let res = wallet::make_sell_create_swap_payload(WALLET_NAME.to_string(), btc)
                    .await
                    .map_err(ips_bs::MakePayloadError::from);

                let _resp = browser.tabs().send_message(
                    tab_id,
                    JsValue::from_serde(&ips_bs::ToPage::SellResponse(res)).unwrap(),
                    JsValue::null(),
                );
            });
        }
        ips_bs::ToBackground::BuyRequest(usdt) => {
            spawn_local(async move {
                let tab_id = message_sender.tab.expect("tab id to exist").id;

                let res = wallet::make_buy_create_swap_payload(WALLET_NAME.to_string(), usdt)
                    .await
                    .map_err(ips_bs::MakePayloadError::from);

                let _resp = browser.tabs().send_message(
                    tab_id,
                    JsValue::from_serde(&ips_bs::ToPage::BuyResponse(res)).unwrap(),
                    JsValue::null(),
                );
            });
        }
        ips_bs::ToBackground::SignRequest(tx_hex) => spawn_local(async move {
            let tab_id = message_sender.tab.expect("tab id to exist").id;

            let res = wallet::extract_trade(WALLET_NAME.to_string(), tx_hex.clone()).await;

            match res {
                Ok(trade) => {
                    log::debug!("Extracted trade: {:?}", trade);

                    let tx_data = TransactionData {
                        hex: tx_hex,
                        decoded: trade,
                        tab_id,
                    };

                    let mut guard = SIGN_STATE.lock().await;
                    *guard = SignState::Trade(tx_data);

                    increment_badge_counter().await;
                }
                Err(e) => {
                    let _resp = browser.tabs().send_message(
                        tab_id,
                        JsValue::from_serde(&ips_bs::SignAndSendError::ExtractTrade(format!(
                            "{:#}",
                            e
                        )))
                        .unwrap(),
                        JsValue::null(),
                    );
                }
            }
        }),
        ips_bs::ToBackground::NewAddress => {
            spawn_local(async move {
                let tab_id = message_sender.tab.expect("tab id to exist").id;
                let response = wallet::get_address(WALLET_NAME.to_string())
                    .await
                    .map_err(|e| ips_bs::NewAddressError(format!("{:#}", e)));

                let _resp = browser.tabs().send_message(
                    tab_id,
                    JsValue::from_serde(&ips_bs::ToPage::NewAddressResponse(response)).unwrap(),
                    JsValue::null(),
                );
            });
        }
        ips_bs::ToBackground::LoanRequest(collateral) => {
            spawn_local(async move {
                let tab_id = message_sender.tab.expect("tab id to exist").id;
                let res = wallet::make_loan_request(WALLET_NAME.to_string(), collateral)
                    .await
                    .map_err(ips_bs::MakeLoanRequestError::from);

                let _resp = browser.tabs().send_message(
                    tab_id,
                    JsValue::from_serde(&ips_bs::ToPage::LoanRequestResponse(Box::new(res)))
                        .unwrap(),
                    JsValue::null(),
                );
            });
        }
        ips_bs::ToBackground::SignLoan(loan_response) => spawn_local(async move {
            let tab_id = message_sender.tab.expect("tab id to exist").id;

            let res = wallet::extract_loan(WALLET_NAME.to_string(), loan_response).await;

            match res {
                Ok(loan_details) => {
                    log::debug!("Extracted loan details: {:?}", loan_details);

                    let loan_data = LoanData {
                        details: loan_details,
                        tab_id,
                    };

                    let mut guard = SIGN_STATE.lock().await;
                    *guard = SignState::Loan(loan_data);

                    increment_badge_counter().await;
                }
                Err(e) => {
                    let _resp = browser.tabs().send_message(
                        tab_id,
                        JsValue::from_serde(&ips_bs::SignAndSendError::ExtractTrade(format!(
                            "{:#}",
                            e
                        )))
                        .unwrap(),
                        JsValue::null(),
                    );
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

async fn background_status(name: String, sign_state: SignState) -> Result<BackgroundStatus> {
    let status = wallet::wallet_status(name).await?;
    if let wallet::WalletStatus {
        exists: false,
        loaded: false,
    } = status
    {
        log::debug!("Wallet does not exist");
        return Ok(BackgroundStatus::new(WalletStatus::default(), sign_state));
    }

    if let wallet::WalletStatus {
        exists: true,
        loaded: false,
    } = status
    {
        log::debug!("Wallet exists but not loaded");
        return Ok(BackgroundStatus::new(WalletStatus::NotLoaded, sign_state));
    }

    if let wallet::WalletStatus {
        exists: false,
        loaded: true,
    } = status
    {
        bail!("incorrect state: wallet cannot be loaded if it doesn't exist")
    }

    let address = wallet::get_address(WALLET_NAME.to_string())
        .await
        .context("could not get address")?;

    Ok(BackgroundStatus::new(
        WalletStatus::Loaded {
            address: address.to_string(),
        },
        sign_state,
    ))
}

#[derive(Debug, Deserialize)]
struct MessageSender {
    tab: Option<Tab>,
}

#[derive(Debug, Deserialize)]
struct Tab {
    id: u32,
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
