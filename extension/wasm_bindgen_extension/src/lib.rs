use js_sys::{Function, Object, Promise};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Browser;

    pub static browser: Browser;

    #[wasm_bindgen(method, getter)]
    pub fn windows(this: &Browser) -> Windows;

    #[wasm_bindgen(method, getter)]
    pub fn runtime(this: &Browser) -> Runtime;

    #[wasm_bindgen(method, getter)]
    pub fn tabs(this: &Browser) -> Tabs;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Tabs;

    #[wasm_bindgen(method)]
    pub fn query(this: &Tabs, info: &Object) -> Promise;

    #[wasm_bindgen(method, js_name = sendMessage)]
    pub fn send_message(this: &Tabs, id: u32, msg: JsValue, options: JsValue) -> Promise;
}

#[derive(Serialize)]
pub struct QueryObject {
    #[serde(rename = "currentWindow")]
    pub current_window: bool,
    pub active: bool,
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Tab;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &Tab) -> u32;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Windows;

    #[wasm_bindgen(method)]
    pub fn create(this: &Windows, info: &Object) -> Promise;

    #[wasm_bindgen(method)]
    pub fn remove(this: &Windows, window_id: u16) -> Promise;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Runtime;

    #[wasm_bindgen(method, getter, js_name = onMessage)]
    pub fn on_message(this: &Runtime) -> Event;

    #[wasm_bindgen(method, js_name = getURL)]
    pub fn get_url(this: &Runtime, path: String) -> String;

    #[wasm_bindgen(method, js_name = getBackgroundPage)]
    pub fn get_background_page(this: &Runtime) -> Background;

    #[wasm_bindgen(method, js_name = sendMessage)]
    pub fn send_message(
        this: &Runtime,
        extension_id: Option<&str>,
        message: &JsValue,
        options: Option<&Object>,
    ) -> Promise;

    #[wasm_bindgen(method)]
    pub fn connect(this: &Runtime, extension_id: Option<&str>, connect_info: &Object) -> Port;

    #[wasm_bindgen(method, getter, js_name = onConnect)]
    pub fn on_connect(this: &Runtime) -> Event;

}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Event;

    #[wasm_bindgen(method, js_name = addListener)]
    pub fn add_listener(this: &Event, closure: &Function);

    #[wasm_bindgen(method, js_name = removeListener)]
    pub fn remove_listener(this: &Event, closure: &Function);
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Document;

    #[wasm_bindgen(method)]
    pub fn write(this: &Document, content: String);
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Background;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type Port;

    #[wasm_bindgen(method, getter)]
    pub fn name(this: &Port) -> String;

    // TODO is this correct ?
    #[wasm_bindgen(method, getter)]
    pub fn error(this: &Port) -> js_sys::Error;

    #[wasm_bindgen(method)]
    pub fn disconnect(this: &Port);

    #[wasm_bindgen(method, getter, js_name = onDisconnect)]
    pub fn on_disconnect(this: &Port) -> Event;

    #[wasm_bindgen(method, getter, js_name = onMessage)]
    pub fn on_message(this: &Port) -> Event;

    #[wasm_bindgen(method, js_name = postMessage)]
    pub fn post_message(this: &Port, value: &JsValue);

    #[wasm_bindgen(method, getter)]
    pub fn sender(this: &Port) -> Option<MessageSender>;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    pub type MessageSender;

    #[wasm_bindgen(method, getter)]
    pub fn tab(this: &MessageSender) -> Option<Tab>;

    // TODO is this correct ?
    #[wasm_bindgen(method, getter, js_name = frameId)]
    pub fn frame_id(this: &MessageSender) -> Option<u32>;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &MessageSender) -> Option<String>;

    #[wasm_bindgen(method, getter)]
    pub fn url(this: &MessageSender) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name = tlsChannelId)]
    pub fn tls_channel_id(this: &MessageSender) -> Option<String>;
}

#[macro_export]
macro_rules! object {
    ($($key:literal: $value:expr,)*) => {{
        let obj: js_sys::Object = js_sys::Object::new();
        // TODO make this more efficient
        $(wasm_bindgen::UnwrapThrowExt::unwrap_throw(js_sys::Reflect::set(
            &obj,
            &wasm_bindgen::JsValue::from(wasm_bindgen::intern($key)),
            &wasm_bindgen::JsValue::from($value),
        ));)*
        obj
    }};
}
