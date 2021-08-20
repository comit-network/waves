//! Rust Bindings for the browser's global `runtime` namespace that is available to extensions.

use js_sys::Function;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Event;

    #[wasm_bindgen(method, js_name = addListener)]
    pub fn add_listener(this: &Event, callback: &Function);
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Runtime;

    #[wasm_bindgen(method, getter, js_name = onMessage)]
    pub fn on_message(this: &Runtime) -> Event;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Browser;

    pub static browser: Browser;

    #[wasm_bindgen(method, getter)]
    pub fn runtime(this: &Browser) -> Runtime;
}
