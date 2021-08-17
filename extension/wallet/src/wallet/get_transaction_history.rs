use anyhow::Result;
use elements::Txid;
use futures::lock::Mutex;

use crate::{
    wallet::{current, Wallet},
    ESPLORA_CLIENT,
};
use wasm_bindgen::UnwrapThrowExt;

pub async fn get_transaction_history(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<Vec<Txid>> {
    let client = ESPLORA_CLIENT.lock().expect_throw("can get lock");

    let wallet = current(&name, current_wallet).await?;

    // We have a single address, so looking for the transaction
    // history of said address is sufficient
    let address = wallet.get_address();
    let history = client.fetch_transaction_history(&address).await?;

    Ok(history)
}
