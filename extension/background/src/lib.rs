use futures::{Future, FutureExt};
use js_sys::Promise;
use message_types::{bs_ps, cs_bs, Component};
use serde::{Deserialize, Serialize};
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

    let msg: message_types::Message = match js_value.clone().into_serde() {
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
                Ok(msg) => return handle_msg_from_ps(msg),
                Err(_) => {
                    log::warn!("BS: Unexpected message: {:?}", js_value);
                    // TODO introduce error message
                    return Promise::resolve(&JsValue::from_str("Unknown request"));
                }
            }
        }
        (Component::Background, Component::Content) => {
            match (js_value.into_serde(), message_sender.into_serde()) {
                (Ok(msg), Ok(sender)) => return handle_msg_from_cs(msg, sender),
                (_, _) => {
                    log::warn!("BS: Unexpected message: {:?}", js_value);
                    // TODO introduce error message
                    return Promise::resolve(&JsValue::from_str("Unknown request"));
                }
            }
        }
        (_, source) => {
            log::warn!("BS: Unexpected message from {:?}", source);
            // TODO introduce error message
            return Promise::resolve(&JsValue::from_str("Unknown request"));
        }
    }
}

fn handle_msg_from_ps(msg: bs_ps::Message) -> Promise {
    log::info!("BS: Received message from Popup: {:?}", msg);
    // TODO only needed to send something all the way back to PS, e.g. signed data
    let _tab_id = msg.content_tab_id.clone();
    match msg.rpc_data {
        bs_ps::RpcData::WalletStatus => {
            log::debug!("Received status request from PS.");
            future_to_promise(wallet::wallet_status(WALLET_NAME.to_string()))
        }
        bs_ps::RpcData::CreateWallet(name, password) => {
            log::debug!("Creating wallet: {} ", name);
            future_to_promise(
                wallet::create_new_wallet(name.clone(), password.clone())
                    .then(move |_| wallet::wallet_status(name.clone())),
            )
        }
        bs_ps::RpcData::UnlockWallet(name, password) => {
            // TODO unlock wallet
            log::debug!("Received unlock request from PS");
            future_to_promise(
                wallet::load_existing_wallet(name.clone(), password.clone())
                    .then(move |_| wallet::wallet_status(name.clone())),
            )
        }
        bs_ps::RpcData::Hello(data) => {
            // TODO this was just demo and should go away, for now, we keep it here
            // for testing if the whole communication chain still works
            log::error!("Currently not supported {:?}", data);
            Promise::resolve(&JsValue::from_str("UNKNOWN"))
        }
    }
}

fn handle_msg_from_cs(msg: cs_bs::Message, message_sender: MessageSender) -> Promise {
    log::info!("BS: Received from CS: {:?}", &msg);

    // let popup = Popup {
    //     url: format!(
    //         "popup.html?content_tab_id={}",
    //         message_sender.tab.expect("tab id to exist").id
    //     ),
    //     type_: "popup".to_string(),
    //     height: 200,
    //     width: 200,
    // };
    // let js_value = JsValue::from_serde(&popup).unwrap();
    // let object = Object::try_from(&js_value).unwrap();
    // let popup_window = browser.windows().create(&object);
    //
    // log::info!("Popup created {:?}", popup_window);
    // // TODO proper response
    match msg.rpc_data {
        cs_bs::RpcData::GetBalance => {
            spawn_local(async move {
                let result = wallet::get_balances(WALLET_NAME.to_string()).await;
                let tab_id = message_sender.tab.expect("tab id to exist").id;

                match result {
                    Ok(_array) => {
                        //TODO deserialize array and serialize again ? dafuq
                        log::debug!("Received balance info {:?}", _array);
                        let _resp = browser.tabs().send_message(
                            tab_id,
                            JsValue::from_serde(&cs_bs::Message {
                                rpc_data: cs_bs::RpcData::Balance,
                                target: Component::Content,
                                source: Component::Background,
                            })
                            .unwrap(),
                            JsValue::null(),
                        );
                    }
                    Err(err) => {
                        log::error!("Could not get balance {:?}", err);
                    }
                }
            });
            // TODO: how should we deal with this? This is a response to PS
        }
        _ => {}
    }

    return Promise::resolve(&JsValue::from("OK"));
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
