use message_types::{cs_bs, ips_cs};
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

fn handle_msg_from_bs(msg_in: JsValue) {
    // TODO: Filter different messages

    let window = web_sys::window().expect("no global `window` exists");
    log::info!("CS: Received response from BS: {:?}", msg_in);

    if let Ok(msg_in) = msg_in.into_serde::<cs_bs::Message>() {
        let msg_out = ips_cs::Message::from(msg_in);
        log::info!("CS: Sending response to IPS: {:?}", msg_out);
        window
            .post_message(&JsValue::from_serde(&msg_out).unwrap(), "*")
            .unwrap();
    }
}

fn handle_msg_from_ips(msg_in: JsValue) {
    let msg_in: MessageEvent = msg_in.into();
    let msg_in: JsValue = msg_in.data();

    log::info!("CS: received from IPS: {:?}", msg_in);

    if let Ok(msg_in) = msg_in.into_serde::<ips_cs::Message>() {
        let msg_out = cs_bs::Message::from(msg_in);
        log::debug!("CS: Sending message BS: {:?}", msg_out);
        let js_value = JsValue::from_serde(&msg_out).unwrap();

        // TODO: Handle error response?
        let _resp = browser.runtime().send_message(None, &js_value, None);
    }
}
