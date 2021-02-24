use message_types::cs_bs;
use std::future::Future;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_extension::browser;
use wasm_bindgen_futures::spawn_local;
use web_sys::MessageEvent;

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    log::info!("CS: Hello World");

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let head = document.head().expect("document should have a body");

    let url = browser.runtime().get_url("js/in_page.js".to_string());

    // Create new script tag
    let script_tag = document.create_element("script").unwrap();
    script_tag.set_attribute("src", &url).unwrap();
    script_tag.set_attribute("type", "module").unwrap();

    // add script to the top
    let first_child = head.first_child();
    head.insert_before(&script_tag, first_child.as_ref())
        .unwrap();

    // create listener to receive messages from in-page script
    let cb_handle_msg_from_ips = Closure::wrap(Box::new(handle_msg_from_ips) as Box<dyn Fn(_)>);
    window
        .add_event_listener_with_callback(
            "message",
            cb_handle_msg_from_ips.as_ref().unchecked_ref(),
        )
        .unwrap();
    cb_handle_msg_from_ips.forget();

    // create listener to receive messages from background script
    let handle_msg_from_bs = Closure::wrap(Box::new(handle_msg_from_bs) as Box<dyn Fn(_)>);
    browser
        .runtime()
        .on_message()
        .add_listener(handle_msg_from_bs.as_ref().unchecked_ref());
    handle_msg_from_bs.forget();

    Ok(())
}

fn handle_msg_from_bs(msg: JsValue) {
    // TODO: Filter different messages

    let window = web_sys::window().expect("no global `window` exists");
    log::info!("CS: Received response from BS: {:?}", msg);
    let msg: cs_bs::Message = msg.into_serde().unwrap();

    window
        .post_message(
            &JsValue::from_serde(&cs_bs::Message {
                data: msg.data,
                target: "in-page".to_string(),
                source: "content".to_string(),
            })
            .unwrap(),
            "*",
        )
        .unwrap();
}

fn handle_msg_from_ips(msg: JsValue) {
    let msg: MessageEvent = msg.into();
    let js_value: JsValue = msg.data();
    // TODO: Actually only accept messages from IPS
    if let Some(string) = js_value.as_string() {
        log::info!("CS: Received from IPS: {:?}", string);
        let msg = cs_bs::Message {
            data: string,
            target: "background".to_string(),
            source: "content".to_string(),
        };
        // sending message to Background script
        let js_value = JsValue::from_serde(&msg).unwrap();

        // TODO: Handle error response?
        let _resp = browser.runtime().send_message(js_value);
    }
}

pub async fn unwrap_future<F>(future: F)
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
