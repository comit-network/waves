mod document;
mod port;
mod runtime;
mod tabs;
mod windows;

use crate::{runtime::Runtime, tabs::Tabs, windows::Windows};
use js_sys::{Function, Object};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub struct QueryObject {
    #[serde(rename = "currentWindow")]
    pub current_window: bool,
    pub active: bool,
}

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

    #[wasm_bindgen(method, getter, js_name = browserAction)]
    pub fn browser_action(this: &Browser) -> BrowserAction;

}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type BrowserAction;

    /// https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/browserAction/setBadgeText
    #[wasm_bindgen(method, js_name = setBadgeText)]
    pub fn set_badge_text(this: &BrowserAction, badge_text: &Object);

    /// https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/browserAction/setBadgeBackgroundColor
    #[wasm_bindgen(method, js_name = setBadgeBackgroundColor)]
    pub fn set_badge_background_color(this: &BrowserAction, badge_color: &Object);

    /// https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/browserAction/setBadgeBackgroundColor
    #[wasm_bindgen(method, js_name = setBadgeTextColor)]
    pub fn set_badge_text_color(this: &BrowserAction, badge_color: &Object);
}

#[wasm_bindgen]
extern "C" {
    pub type Event;

    #[wasm_bindgen(method, js_name = addListener)]
    pub fn add_listener(this: &Event, callback: &Function);

    #[wasm_bindgen(method, js_name = removeListener)]
    pub fn remove_listener(this: &Event, callback: &Function);

    #[wasm_bindgen(method, js_name = hasListener)]
    pub fn has_listener(this: &Event, callback: &Function) -> bool;

    #[wasm_bindgen(method, js_name = hasListeners)]
    pub fn has_listeners(this: &Event) -> bool;
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
