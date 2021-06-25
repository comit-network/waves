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

#[wasm_bindgen]
pub async fn get_balances(name: String) -> Result<JsValue, JsValue> {
    let balances = map_err_from_anyhow!(wallet::get_balances(name).await)?;
    let balances = map_err_from_anyhow!(JsValue::from_serde(&balances))?;

    Ok(balances)
}

#[wasm_bindgen]
pub async fn make_sell_create_swap_payload(name: String, btc: String) -> Result<JsValue, JsValue> {
    let payload = map_err_from_anyhow!(wallet::make_sell_create_swap_payload(name, btc).await)?;
    let payload = map_err_from_anyhow!(JsValue::from_serde(&payload))?;

    Ok(payload)
}

#[wasm_bindgen]
pub async fn make_buy_create_swap_payload(name: String, usdt: String) -> Result<JsValue, JsValue> {
    let payload = map_err_from_anyhow!(wallet::make_buy_create_swap_payload(name, usdt).await)?;
    let payload = map_err_from_anyhow!(JsValue::from_serde(&payload))?;

    Ok(payload)
}

#[wasm_bindgen]
pub async fn make_loan_request_payload(
    name: String,
    collateral: String,
) -> Result<JsValue, JsValue> {
    let payload = map_err_from_anyhow!(wallet::make_loan_request(name, collateral).await)?;
    let payload = map_err_from_anyhow!(JsValue::from_serde(&payload))?;

    Ok(payload)
}

#[wasm_bindgen]
pub async fn sign_and_send(name: String, tx_hex: String) -> Result<JsValue, JsValue> {
    let txid = map_err_from_anyhow!(wallet::sign_and_send_swap_transaction(name, tx_hex).await)?;
    let txid = map_err_from_anyhow!(JsValue::from_serde(&txid))?;

    Ok(txid)
}

#[wasm_bindgen]
pub async fn extract_trade(name: String, tx_hex: String) -> Result<JsValue, JsValue> {
    let trade = map_err_from_anyhow!(wallet::extract_trade(name, tx_hex).await)?;
    let trade = map_err_from_anyhow!(JsValue::from_serde(&trade))?;

    Ok(trade)
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
