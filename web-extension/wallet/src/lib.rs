use wasm_bindgen::prelude::*;

const WALLET_NAME: &str = "demo-wallet";

#[wasm_bindgen]
pub async fn wallet_status() -> Result<JsValue, JsValue> {
    let status = map_err_from_anyhow!(wallet::wallet_status(WALLET_NAME.to_string()).await)?;

    let status = map_err_from_anyhow!(JsValue::from_serde(&status))?;

    Ok(status)
}

#[wasm_bindgen]
pub async fn get_address() -> Result<JsValue, JsValue> {
    let address = map_err_from_anyhow!(wallet::get_address(WALLET_NAME.to_string()).await)?;
    let address = map_err_from_anyhow!(JsValue::from_serde(&address))?;

    Ok(address)
}

#[macro_export]
macro_rules! map_err_from_anyhow {
    ($e:expr) => {
        match $e {
            Ok(i) => Ok(i),
            Err(e) => Err(JsValue::from_str(&format!("{:#}", e))),
        }
    };
}
