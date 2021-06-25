use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn wallet_status(wallet_name: String) -> Result<JsValue, JsValue> {
    let status = map_err_from_anyhow!(wallet::wallet_status(wallet_name).await)?;

    let status = map_err_from_anyhow!(JsValue::from_serde(&status))?;

    Ok(status)
}

#[wasm_bindgen]
pub async fn get_address(wallet_name: String) -> Result<JsValue, JsValue> {
    let address = map_err_from_anyhow!(wallet::get_address(wallet_name).await)?;
    let address = map_err_from_anyhow!(JsValue::from_serde(&address))?;

    Ok(address)
}

#[wasm_bindgen]
pub async fn create_new_wallet(name: String, password: String) -> Result<(), JsValue> {
    Ok(map_err_from_anyhow!(
        wallet::create_new_wallet(name, password).await
    )?)
}

#[wasm_bindgen]
pub async fn unlock_wallet(name: String, password: String) -> Result<(), JsValue> {
    Ok(map_err_from_anyhow!(
        wallet::load_existing_wallet(name, password).await
    )?)
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
