// TODO: Make this configurable again. We used to be able to set and
// read this using local storage, but after re-working the extension
// it is no longer the case
pub fn try_init() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
}
