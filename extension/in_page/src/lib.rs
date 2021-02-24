extern crate console_error_panic_hook;
use futures::channel::oneshot;
use js_sys::{global, Object, Promise};
use message_types::ips_bs::{ToBackground, ToPage};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::future_to_promise;
use web_sys::MessageEvent;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    log::info!("IPS: Hello World");

    let global: Object = global();

    let boxed = Box::new(wallet_status) as Box<dyn Fn() -> Promise>;
    let closure = Closure::wrap(boxed).into_js_value();
    js_sys::Reflect::set(&global, &JsValue::from("wallet_status"), &closure).unwrap();

    let boxed = Box::new(get_sell_create_swap_payload) as Box<dyn Fn(String) -> Promise>;
    let closure = Closure::wrap(boxed).into_js_value();
    js_sys::Reflect::set(
        &global,
        &JsValue::from("get_sell_create_swap_payload"),
        &closure,
    )
    .unwrap();

    let boxed = Box::new(get_buy_create_swap_payload) as Box<dyn Fn(String) -> Promise>;
    let closure = Closure::wrap(boxed).into_js_value();
    js_sys::Reflect::set(
        &global,
        &JsValue::from("get_buy_create_swap_payload"),
        &closure,
    )
    .unwrap();

    let boxed = Box::new(sign_and_send) as Box<dyn Fn(String) -> Promise>;
    let closure = Closure::wrap(boxed).into_js_value();
    js_sys::Reflect::set(&global, &JsValue::from("sign_and_send"), &closure).unwrap();

    let window = web_sys::window().expect("no global `window` exists");
    let js_value = JsValue::from("IPS_injected");
    window.post_message(&js_value, "*").unwrap();
}

macro_rules! send_to_cs {
    ($js_value:ident, $msg_type:path) => {{
        let (sender, receiver) = oneshot::channel::<Result<JsValue, JsValue>>();
        let mut sender = Some(sender);
        let mut listener = create_listener!(sender, $msg_type);

        let window = web_sys::window().expect("no global `window` exists");
        window.post_message(&$js_value, "*").unwrap();

        let fut = async move {
            let response = receiver.await;
            let response = response.map_err(|_| JsValue::from_str("IPS: No response from CS"))?;

            listener.remove();
            response
        };

        future_to_promise(fut)
    }};
}

macro_rules! create_listener {
    ($sender:expr, $msg_type:path) => {{
        let func = move |msg: MessageEvent| {
            let msg = msg.data();

            if let Ok(msg) = msg.into_serde::<ToPage>() {
                log::info!("IPS: Received response from CS: {:?}", msg);
                if let $msg_type(payload) = msg {
                    let msg = payload
                        .map(|res| JsValue::from_serde(&res).unwrap())
                        .map_err(|rej| JsValue::from_serde(&rej).unwrap());

                    $sender
                        .take()
                        .expect("only send response once")
                        .send(msg)
                        .unwrap();
                }
            }
        };

        let cb = Closure::wrap(Box::new(func) as Box<dyn FnMut(MessageEvent)>);
        Listener::new("message".to_string(), cb)
    }};
}

#[wasm_bindgen]
pub fn wallet_status() -> Promise {
    let msg = JsValue::from_serde(&ToBackground::WalletStatusRequest).unwrap();
    send_to_cs!(msg, ToPage::StatusResponse)
}

#[wasm_bindgen]
pub fn get_sell_create_swap_payload(btc: String) -> Promise {
    let msg = JsValue::from_serde(&ToBackground::SellRequest(btc)).unwrap();
    send_to_cs!(msg, ToPage::SellResponse)
}

#[wasm_bindgen]
pub fn get_buy_create_swap_payload(usdt: String) -> Promise {
    let msg = JsValue::from_serde(&ToBackground::BuyRequest(usdt)).unwrap();
    send_to_cs!(msg, ToPage::BuyResponse)
}

#[wasm_bindgen]
pub fn sign_and_send(tx_hex: String) -> Promise {
    let msg = JsValue::from_serde(&ToBackground::SignRequest(tx_hex)).unwrap();
    send_to_cs!(msg, ToPage::SignResponse)
}

struct Listener<F>
where
    F: ?Sized,
{
    name: String,
    cb: Closure<F>,
}

impl<F> Listener<F>
where
    F: ?Sized,
{
    fn new(name: String, cb: Closure<F>) -> Self
    where
        F: FnMut(MessageEvent) + 'static,
    {
        let window = web_sys::window().expect("no global `window` exists");
        window
            .add_event_listener_with_callback(&name, cb.as_ref().unchecked_ref())
            .unwrap();

        Self { name, cb }
    }

    fn remove(&mut self) {
        let window = web_sys::window().expect("no global `window` exists");
        window
            .remove_event_listener_with_callback(&self.name, self.cb.as_ref().unchecked_ref())
            .unwrap();
    }
}
