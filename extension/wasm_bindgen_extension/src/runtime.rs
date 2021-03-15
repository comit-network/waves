use crate::{port::Port, Event};
use js_sys::{Object, Promise};
use wasm_bindgen::{prelude::*, JsValue};

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
    pub type Background;
}
