use message_types::ips_bs::{ToBackground, ToPage};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_extension::browser;
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
    let window = web_sys::window().expect("no global `window` exists");

    log::trace!("CS: Received response from BS: {:?}", msg);

    if let Ok(msg) = msg.into_serde::<ToPage>() {
        log::debug!("CS: Forwarding response from BS to IPS: {:?}", msg);
        window
            .post_message(&JsValue::from_serde(&msg).unwrap(), "*")
            .unwrap();
    }
}

fn handle_msg_from_ips(msg: JsValue) {
    let event = MessageEvent::from(msg);
    let msg = event.data();

    log::trace!("CS: Received message from IPS: {:?}", msg);

    if let Ok(msg) = msg.into_serde::<ToBackground>() {
        log::debug!("CS: Forwarding message from IPS to BS: {:?}", msg);
        let msg = JsValue::from_serde(&msg).unwrap();

        // the background script will respond by sending a new
        // message, so we can ignore this
        let _ = browser.runtime().send_message(None, &msg, None);
    }
}
