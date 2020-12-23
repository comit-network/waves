use futures::lock::Mutex;
use wasm_bindgen::JsValue;

use elements_fun::Address;

use crate::wallet::{current, Wallet};

pub async fn get_address(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<Address, JsValue> {
    let wallet = current(&name, current_wallet).await?;

    let address = wallet.get_address()?;

    Ok(address)
}
