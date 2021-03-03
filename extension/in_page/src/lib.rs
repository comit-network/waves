use anyhow::{Context, Result};
use elements::{encode::deserialize, Txid};
extern crate console_error_panic_hook;
use futures::{channel::mpsc, StreamExt};
use js_sys::{global, Object, Promise};
use message_types::{ips_cs, ips_cs::RpcData, Component};
use std::future::Future;
use wallet::CreateSwapPayload;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{future_to_promise, spawn_local};
use web_sys::MessageEvent;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    log::info!("IPS: Hello World");

    let global: Object = global();

    let boxed = Box::new(balances) as Box<dyn Fn() -> Promise>;
    let closure = Closure::wrap(boxed).into_js_value();
    js_sys::Reflect::set(&global, &JsValue::from("balances"), &closure).unwrap();

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
pub fn balances() -> Promise {
    let (mut sender, mut receiver) = mpsc::channel::<JsValue>(10);
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
            if let RpcData::Balance(balance_entry) = rpc_data {
                sender
                    .try_send(JsValue::from_serde(balance_entry).unwrap())
                    .unwrap();
            }
        }
    };

    let cb = Closure::wrap(Box::new(func) as Box<dyn FnMut(MessageEvent)>);
    let listener = Listener::new("message".to_string(), cb);

    let window = web_sys::window().expect("no global `window` exists");
    let js_value = JsValue::from_serde(&ips_cs::Message {
        rpc_data: ips_cs::RpcData::GetBalance,
        target: Component::Content,
        source: Component::InPage,
    })
    .unwrap();
    window.post_message(&js_value, "*").unwrap();

    let fut = async move {
        let response = receiver.next().await;
        let response = response.ok_or_else(|| JsValue::from_str("IPS: No response from CS"))?;

        drop(listener);
        Ok(response)
    };

    future_to_promise(fut)
}

#[wasm_bindgen]
pub fn wallet_status() -> Promise {
    let (mut sender, mut receiver) = mpsc::channel::<JsValue>(10);
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
            if let RpcData::WalletStatus(wallet_status) = rpc_data {
                sender
                    .try_send(JsValue::from_serde(wallet_status).unwrap())
                    .unwrap();
            }
        }
    };

    let cb = Closure::wrap(Box::new(func) as Box<dyn FnMut(MessageEvent)>);
    let listener = Listener::new("message".to_string(), cb);

    let window = web_sys::window().expect("no global `window` exists");
    let js_value = JsValue::from_serde(&ips_cs::Message {
        rpc_data: ips_cs::RpcData::GetWalletStatus,
        target: Component::Content,
        source: Component::InPage,
    })
    .unwrap();
    window.post_message(&js_value, "*").unwrap();

    let fut = async move {
        let response = receiver.next().await;
        let response = response.ok_or_else(|| JsValue::from_str("IPS: No response from CS"))?;

        drop(listener);
        Ok(response)
    };

    future_to_promise(fut)
}

#[wasm_bindgen]
pub fn get_sell_create_swap_payload(btc: String) -> Promise {
    let (mut sender, mut receiver) = mpsc::channel::<CreateSwapPayload>(10);
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
            if let RpcData::SellCreateSwapPayload(payload) = rpc_data.clone() {
                sender.try_send(payload).unwrap();
            }
        }
    };

    let cb = Closure::wrap(Box::new(func) as Box<dyn FnMut(MessageEvent)>);
    let listener = Listener::new("message".to_string(), cb);

    let window = web_sys::window().expect("no global `window` exists");
    let js_value = JsValue::from_serde(&ips_cs::Message {
        rpc_data: ips_cs::RpcData::GetSellCreateSwapPayload(btc),
        target: Component::Content,
        source: Component::InPage,
    })
    .unwrap();
    window.post_message(&js_value, "*").unwrap();

    let fut = async move {
        let response = receiver.next().await;
        let response = response.ok_or_else(|| JsValue::from_str("IPS: No response from CS"))?;
        let response = JsValue::from_serde(&response).unwrap();

        drop(listener);
        Ok(response)
    };

    future_to_promise(fut)
}

#[wasm_bindgen]
pub fn get_buy_create_swap_payload(usdt: String) -> Promise {
    log::debug!("get_buy_create_swap_payload");

    let (mut sender, mut receiver) = mpsc::channel::<CreateSwapPayload>(10);
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
            if let RpcData::BuyCreateSwapPayload(payload) = rpc_data.clone() {
                sender.try_send(payload).unwrap();
            }
        }
    };

    let cb = Closure::wrap(Box::new(func) as Box<dyn FnMut(MessageEvent)>);
    let listener = Listener::new("message".to_string(), cb);

    let window = web_sys::window().expect("no global `window` exists");
    let js_value = JsValue::from_serde(&ips_cs::Message {
        rpc_data: ips_cs::RpcData::GetBuyCreateSwapPayload(usdt),
        target: Component::Content,
        source: Component::InPage,
    })
    .unwrap();
    window.post_message(&js_value, "*").unwrap();

    let fut = async move {
        let response = receiver.next().await;
        let response = response.ok_or_else(|| JsValue::from_str("IPS: No response from CS"))?;
        let response = JsValue::from_serde(&response).unwrap();

        drop(listener);
        Ok(response)
    };

    future_to_promise(fut)
}

#[wasm_bindgen]
pub fn sign_and_send(tx_hex: String) -> Promise {
    let (mut sender, mut receiver) = mpsc::channel::<Txid>(10);
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
            if let RpcData::SwapTxid(txid) = rpc_data {
                sender.try_send(*txid).unwrap();
            }
        }
    };

    let cb = Closure::wrap(Box::new(func) as Box<dyn FnMut(MessageEvent)>);
    let listener = Listener::new("message".to_string(), cb);

    let window = web_sys::window().expect("no global `window` exists");
    let js_value = JsValue::from_serde(&ips_cs::Message {
        rpc_data: ips_cs::RpcData::SignAndSend(tx_hex),
        target: Component::Content,
        source: Component::InPage,
    })
    .unwrap();
    window.post_message(&js_value, "*").unwrap();

    let fut = async move {
        let response = receiver.next().await;
        let response = response.ok_or_else(|| JsValue::from_str("IPS: No response from CS"))?;
        let response = JsValue::from_serde(&response).unwrap();

        drop(listener);
        Ok(response)
    };

    future_to_promise(fut)
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
}

impl<F> Drop for Listener<F>
where
    F: ?Sized,
{
    fn drop(&mut self) {
        let window = web_sys::window().expect("no global `window` exists");
        window
            .remove_event_listener_with_callback(&self.name, self.cb.as_ref().unchecked_ref())
            .unwrap();
    }
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
