extern crate console_error_panic_hook;
use futures::{channel::mpsc, StreamExt};
use js_sys::{global, Object, Promise};
use message_types::{ips_cs, Component};
use std::future::Future;
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

    let boxed = Box::new(call_backend) as Box<dyn Fn(String) -> Promise>;
    let add_closure = Closure::wrap(boxed);
    let add_closure = add_closure.into_js_value();

    js_sys::Reflect::set(&global, &JsValue::from("call_backend"), &add_closure).unwrap();
}

#[wasm_bindgen]
pub fn call_backend(txt: String) -> Promise {
    let js_value = JsValue::from(txt);

    let (mut sender, mut receiver) = mpsc::channel::<JsValue>(10);
    // create listener
    let func = move |msg: MessageEvent| {
        let js_value: JsValue = msg.data();

        let message: Result<ips_cs::Message, _> = js_value.into_serde();
        if let Ok(ips_cs::Message {
            target,
            data,
            source,
        }) = &message
        {
            match (target, source) {
                (Component::InPage, Component::Content) => {}
                (_, _) => {
                    log::debug!("IPS: Unexpected message from: {:?}", message);
                    return;
                }
            }

            log::info!("IPS: Received response from CS: {:?}", data);
            sender.try_send(JsValue::from_str(&data)).unwrap();
        }
    };

    let cb = Closure::wrap(Box::new(func) as Box<dyn FnMut(MessageEvent)>);
    let listener = Listener::new("message".to_string(), cb);

    let window = web_sys::window().expect("no global `window` exists");
    window.post_message(&js_value, "*").unwrap();

    let fut = async move {
        let response = receiver.next().await;
        let response = response.ok_or_else(|| JsValue::from_str("IPS: No response from CS"))?;

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
