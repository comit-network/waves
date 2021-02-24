use conquer_once::Lazy;
use futures::lock::Mutex;
use js_sys::Object;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_extension::browser;
use wasm_bindgen_futures::spawn_local;

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
    log::info!("BS: Hello World");

    spawn_local(async {
        LOADED_WALLET.lock().await.replace("wallet".to_string());
    });

    let handle_msg_from_cs = Closure::wrap(Box::new(|msg: JsValue, message_sender: JsValue| {
        if !msg.is_object() {
            let log = format!("Invalid request: {:?}", msg);
            log::error!("{}", log);
            return;
        }

        let msg: Message = msg.into_serde().unwrap();
        if msg.target != "background" || msg.source != "content" {
            log::debug!("Unexpected message: {:?}", msg);
            return;
        }

        let message_sender: MessageSender = message_sender.into_serde().unwrap();

        log::info!("BS: Received from CS: {:?}", &msg);

        let popup = Popup {
            url: format!(
                "popup.html?content_tab_id={}",
                message_sender.tab.expect("tab id to exist").id
            ),
            type_: "popup".to_string(),
            height: 200,
            width: 200,
        };
        let js_value = JsValue::from_serde(&popup).unwrap();
        let object = Object::try_from(&js_value).unwrap();
        let popup_window = browser.windows().create(&object);

        log::info!("Popup created {:?}", popup_window);
    }) as Box<dyn Fn(_, _)>);
    browser
        .runtime()
        .on_message()
        .add_listener(handle_msg_from_cs.as_ref().unchecked_ref());
    handle_msg_from_cs.forget();

    // receiving transaction to be signed and published
    let handle_msg_from_ps = Closure::wrap(Box::new(move |js_value: JsValue| {
        if !js_value.is_object() {
            let log = format!("Invalid request: {:?}", js_value);
            log::error!("{}", log);
            return;
        }

        let msg: PopupMessage = match js_value.into_serde() {
            Ok(msg) => msg,
            Err(_) => {
                log::debug!("Unexpected message: {:?}", js_value);
                return;
            }
        };
        if msg.target != "background" || msg.source != "popup" {
            log::debug!("Unexpected message: {:?}", msg);
            return;
        }

        log::info!("Received message from Popup: {:?}", msg);

        spawn_local(async {
            let guard = LOADED_WALLET.lock().await;
            let wallet = guard.as_ref().unwrap();
            log::debug!("Loaded wallet: {}", wallet);

            let _resp = browser.tabs().send_message(
                msg.content_tab_id,
                JsValue::from_serde(&Message {
                    data: msg.data,
                    target: "content".to_string(),
                    source: "background".to_string(),
                })
                .unwrap(),
                JsValue::null(),
            );
        });

        // TODO: Inform PS about success/failure
        // Promise::resolve(&JsValue::from_str("ACK2"))
    }) as Box<dyn FnMut(JsValue)>);

    browser
        .runtime()
        .on_message()
        .add_listener(handle_msg_from_ps.as_ref().unchecked_ref());
    handle_msg_from_ps.forget();
}

#[derive(Serialize, Deserialize)]
struct Popup {
    pub url: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub height: u8,
    pub width: u8,
}

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    data: String,
    target: String,
    source: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct PopupMessage {
    data: String,
    target: String,
    source: String,
    content_tab_id: u32,
}

#[derive(Debug, Deserialize)]
struct PopupWindow {
    id: u16,
}
