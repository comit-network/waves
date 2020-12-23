use crate::{
    storage::Storage,
    wallet::{ListOfWallets, Wallet},
};
use anyhow::Result;
use futures::lock::Mutex;
use wasm_bindgen::JsValue;

pub async fn load_existing(
    name: String,
    password: String,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<(), JsValue> {
    let mut guard = current_wallet.lock().await;

    if let Some(Wallet { name: loaded, .. }) = &*guard {
        return Err(JsValue::from_str(&format!(
            "cannot load wallet '{}' because wallet '{}' is currently loaded",
            name, loaded
        )));
    }

    let storage = Storage::local_storage()?;
    let wallets = storage
        .get_item::<ListOfWallets>("wallets")?
        .unwrap_or_default();

    if !wallets.has(&name) {
        return Err(JsValue::from_str(&format!(
            "wallet '{}' does not exist",
            name
        )));
    }

    let stored_password = storage
        .get_item::<String>(&format!("wallets.{}.password", name))?
        .ok_or_else(|| JsValue::from_str("no password stored for wallet"))?;

    scrypt::scrypt_check(&password, &stored_password)
        .map_err(|_| JsValue::from_str(&format!("bad password for wallet '{}'", name)))?;

    let sk_ciphertext = storage
        .get_item::<String>(&format!("wallets.{}.secret_key", name))?
        .ok_or_else(|| JsValue::from_str("no secret key for wallet"))?;

    let wallet = Wallet::initialize_existing(name, password, sk_ciphertext)?;

    guard.replace(wallet);

    log::info!("Wallet successfully loaded");

    Ok(())
}
