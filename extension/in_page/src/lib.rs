use anyhow::Result;
extern crate console_error_panic_hook;
use futures::channel::{oneshot, oneshot::Sender};
use js_sys::{global, Object, Promise};
use message_types::{ips_cs, ips_cs::RpcData, Component};
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

#[wasm_bindgen]
pub fn wallet_status() -> Promise {
    let js_value = JsValue::from_serde(&ips_cs::Message {
        rpc_data: ips_cs::RpcData::GetWalletStatus,
        target: Component::Content,
        source: Component::InPage,
    })
    .unwrap();
    send_to_cs(&js_value)
}

#[wasm_bindgen]
pub fn get_sell_create_swap_payload(btc: String) -> Promise {
    let js_value = JsValue::from_serde(&ips_cs::Message {
        rpc_data: ips_cs::RpcData::GetSellCreateSwapPayload(btc),
        target: Component::Content,
        source: Component::InPage,
    })
    .unwrap();
    send_to_cs(&js_value)
}

#[wasm_bindgen]
pub fn get_buy_create_swap_payload(usdt: String) -> Promise {
    let js_value = JsValue::from_serde(&ips_cs::Message {
        rpc_data: ips_cs::RpcData::GetBuyCreateSwapPayload(usdt),
        target: Component::Content,
        source: Component::InPage,
    })
    .unwrap();
    send_to_cs(&js_value)
}

#[wasm_bindgen]
pub fn sign_and_send(tx_hex: String) -> Promise {
    let js_value = JsValue::from_serde(&ips_cs::Message {
        rpc_data: ips_cs::RpcData::SignAndSend(tx_hex),
        target: Component::Content,
        source: Component::InPage,
    })
    .unwrap();
    send_to_cs(&js_value)
}

fn send_to_cs(js_value: &JsValue) -> Promise {
    let (sender, receiver) = oneshot::channel::<JsValue>();
    let sender = Some(sender);
    let mut listener = create_listener(sender);

    let window = web_sys::window().expect("no global `window` exists");
    window.post_message(js_value, "*").unwrap();

    let fut = async move {
        let response = receiver.await;
        let response = response.map_err(|_| JsValue::from_str("IPS: No response from CS"))?;

        listener.remove();
        Ok(response)
    };

    future_to_promise(fut)
}

fn create_listener(mut sender: Option<Sender<JsValue>>) -> Listener<dyn FnMut(MessageEvent)> {
    // create listener
    let func = move |msg: MessageEvent| {
        let js_value: JsValue = msg.data();

        let message: Result<ips_cs::Message, _> = js_value.into_serde();
        if let Ok(ips_cs::Message {
            target,
            rpc_data,
            source,
        }) = &message
        {
            match (target, source) {
                (Component::InPage, Component::Content) => {}
                (_, _) => {
                    log::warn!("IPS: Unexpected message from: {:?}", message);
                    return;
                }
            }

            log::info!("IPS: Received response from CS: {:?}", rpc_data);
            match rpc_data {
                RpcData::WalletStatus(wallet_status) => {
                    sender
                        .take()
                        .expect("only send response once")
                        .send(JsValue::from_serde(wallet_status).unwrap())
                        .unwrap();
                }
                RpcData::SellCreateSwapPayload(payload) => {
                    sender
                        .take()
                        .expect("only send response once")
                        .send(JsValue::from_serde(&payload).unwrap())
                        .unwrap();
                }
                RpcData::BuyCreateSwapPayload(payload) => {
                    sender
                        .take()
                        .expect("only send response once")
                        .send(JsValue::from_serde(&payload).unwrap())
                        .unwrap();
                }
                RpcData::SwapTxid(txid) => {
                    sender
                        .take()
                        .expect("only send response once")
                        .send(JsValue::from_serde(txid).unwrap())
                        .unwrap();
                }
                rpc_data => {
                    let msg = format!("Received unsupported message from CS: {:?}", rpc_data);
                    sender
                        .take()
                        .expect("only send response once")
                        .send(JsValue::from_str(&msg))
                        .unwrap();
                }
            }
        }
    };

    let cb = Closure::wrap(Box::new(func) as Box<dyn FnMut(MessageEvent)>);
    Listener::new("message".to_string(), cb)
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
