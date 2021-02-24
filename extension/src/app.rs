use js_sys::Promise;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub struct App {
    link: ComponentLink<Self>,
    content_tab_id: u32,
}

pub enum Msg {
    Sign,
}

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    data: String,
    target: String,
    source: String,
    content_tab_id: u32,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let window = web_sys::window().expect("no global `window` exists");

        let url = Url::parse(&window.location().href().unwrap()).unwrap();
        let queries: HashMap<String, String> = url.query_pairs().into_owned().collect();
        let content_tab_id = queries.get("content_tab_id").unwrap();
        let content_tab_id = content_tab_id.parse::<u32>().unwrap();
        log::debug!("Content tab ID = {}", content_tab_id);

        App {
            link,
            content_tab_id,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Sign => {
                let msg = Message {
                    data: "World".to_string(),
                    target: "background".to_string(),
                    source: "popup".to_string(),
                    content_tab_id: self.content_tab_id,
                };
                let js_value = JsValue::from_serde(&msg).unwrap();
                let _resp = browser.runtime().send_message(js_value);
                // TODO: handle response
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        true
    }

    fn view(&self) -> Html {
        html! {
        <div>
            <p>{ "Hello worlds!" }</p>
                <button onclick=self.link.callback(|_| Msg::Sign)>{ "Sign" }</button>
          </div>
        }
    }

    fn rendered(&mut self, _first_render: bool) {}

    fn destroy(&mut self) {}
}

#[wasm_bindgen]
extern "C" {
    pub type Browser;
    pub static browser: Browser;

    #[wasm_bindgen(method, getter)]
    pub fn runtime(this: &Browser) -> Runtime;

}

#[wasm_bindgen]
extern "C" {
    pub type Runtime;

    #[wasm_bindgen(method, js_name = getBackgroundPage)]
    pub fn get_background_page(this: &Runtime) -> Background;

    #[wasm_bindgen(method, js_name = sendMessage)]
    pub fn send_message(this: &Runtime, value: JsValue) -> Promise;
}

#[wasm_bindgen]
extern "C" {
    pub type Background;

    #[wasm_bindgen(method)]
    pub fn is_locked(this: &Background) -> bool;
}
