use js_sys::{Object, Promise};
use wasm_bindgen::{prelude::*, JsValue};

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Tabs;

    #[wasm_bindgen(method)]
    pub fn query(this: &Tabs, info: &Object) -> Promise;

    #[wasm_bindgen(method, js_name = sendMessage)]
    pub fn send_message(this: &Tabs, id: u32, msg: JsValue, options: JsValue) -> Promise;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Tab;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &Tab) -> u32;
}
