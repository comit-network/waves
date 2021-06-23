use log::Level;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hello_world() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    wasm_logger::init(wasm_logger::Config::new(Level::Debug));

    log::info!("Hello World");
}
