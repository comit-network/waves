use conquer_once::Lazy;
use futures::{lock::Mutex, Future, FutureExt};
use js_sys::Promise;
use message_types::{bs_ps, bs_ps::RpcData, cs_bs, Component};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_extension::browser;
use wasm_bindgen_futures::{future_to_promise, spawn_local};

static LOADED_WALLET: Lazy<Mutex<Option<String>>> = Lazy::new(Mutex::default);

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

    spawn_local(async {
        LOADED_WALLET.lock().await.replace("wallet".to_string());
    });

    // instantiate listener to receive messages from content script
    let handle_msg_from_cs = Closure::wrap(Box::new(handle_msg_from_cs) as Box<dyn Fn(_, _)>);
    browser
        .runtime()
        .on_message()
        .add_listener(handle_msg_from_cs.as_ref().unchecked_ref());
    handle_msg_from_cs.forget();

    // instantiate listener to receive messages from popup script
    let handle_msg_from_ps =
        Closure::wrap(Box::new(handle_msg_from_ps) as Box<dyn FnMut(JsValue) -> Promise>);
    browser
        .runtime()
        .on_message()
        .add_listener(handle_msg_from_ps.as_ref().unchecked_ref());
    handle_msg_from_ps.forget();
}

fn handle_msg_from_ps(js_value: JsValue) -> Promise {
    if !js_value.is_object() {
        let log = format!("BS: Invalid request: {:?}", js_value);
        log::error!("{}", log);
        // TODO introduce error message
        return Promise::resolve(&JsValue::from_str("Unknown request"));
    }

    let msg: bs_ps::Message = match js_value.into_serde() {
        Ok(msg) => msg,
        Err(_) => {
            log::debug!("BS: Unexpected message: {:?}", js_value);
            // TODO introduce error message
            return Promise::resolve(&JsValue::from_str("Unknown request"));
        }
    };

    match (&msg.target, &msg.source) {
        (Component::Background, Component::PopUp) => {}
        (_, source) => {
            log::debug!("BS: Unexpected message from {:?}", source);
            // TODO introduce error message
            return Promise::resolve(&JsValue::from_str("Unknown request"));
        }
    }

    log::info!("BS: Received message from Popup: {:?}", msg);
    let tab_id = msg.content_tab_id.clone();
    match msg.rpc_data {
        RpcData::WalletStatus(name) => {
            log::debug!("Received status request from PS: {}", name);
            return future_to_promise(wallet::wallet_status(name));
        }
        RpcData::CreateWallet(name, password) => {
            log::debug!("Creating wallet: {} ", name);
            return future_to_promise(
                wallet::create_new_wallet(name.clone(), password.clone())
                    .then(move |_| wallet::wallet_status(name.clone())),
            );
        }
        RpcData::UnlockWallet(_data) => {
            // TODO unlock wallet
            log::debug!("Received unlock request from PS");
            Promise::resolve(&JsValue::from_str("UNKNOWN"))
        }
        RpcData::Hello(data) => {
            // TODO this was just demo and should go away, for now, we keep it here
            // for testing if the whole communication chain still works
            spawn_local(async move {
                let guard = LOADED_WALLET.lock().await;
                let wallet = guard.as_ref().unwrap();
                log::debug!("BS: Loaded wallet: {}", wallet);

                let _resp = browser.tabs().send_message(
                    tab_id,
                    JsValue::from_serde(&cs_bs::Message {
                        data: data.clone(),
                        target: Component::Content,
                        source: Component::Background,
                    })
                    .unwrap(),
                    JsValue::null(),
                );
            });
            // TODO: how should we deal with this? This is a response to PS
            Promise::resolve(&JsValue::from_str("UNKNOWN"))
        }
    }
}

// TODO combine both message handlers: there should only be one listener on window
fn handle_msg_from_cs(msg: JsValue, message_sender: JsValue) {
    if !msg.is_object() {
        let log = format!("Invalid request: {:?}", msg);
        log::error!("{}", log);
        return;
    }

    // let msg: cs_bs::Message = msg.into_serde().unwrap();
    // match (&msg.target, &msg.source) {
    //     (Component::Background, Component::Content) => {}
    //     (_, _) => {
    //         log::debug!("BS: Unexpected message: {:?}", msg);
    //         return;
    //     }
    // }
    //
    // let message_sender: MessageSender = message_sender.into_serde().unwrap();
    //
    // log::info!("BS: Received from CS: {:?}", &msg);
    //
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
