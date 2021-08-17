use crate::{
    wallet::{current, Wallet, DEFAULT_SAT_PER_VBYTE},
    BTC_ASSET_ID, ESPLORA_CLIENT,
};
use anyhow::{Context, Result};
use elements::{Address, Txid};
use futures::lock::Mutex;
use wasm_bindgen::UnwrapThrowExt;

pub async fn withdraw_everything_to(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    address: Address,
) -> Result<Txid> {
    let client = ESPLORA_CLIENT.lock().expect_throw("can get lock");

    let btc_asset_id = {
        let guard = BTC_ASSET_ID.lock().expect_throw("can get lock");
        *guard
    };

    let mut wallet = current(&name, current_wallet).await?;
    wallet.sync(&*client).await?;

    let fee_estimates = client.get_fee_estimates().await?;
    let fee_rate = fee_estimates.b_6.unwrap_or_else(|| {
        let default_fee_rate = DEFAULT_SAT_PER_VBYTE;
        log::info!(
            "fee estimate for block target '6' unavailable, falling back to default fee {}",
            default_fee_rate
        );

        default_fee_rate as f32
    }); // try to get into the next 6 blocks;

    let transaction = wallet.withdraw_everything_to_transaction(address, btc_asset_id, fee_rate)?;

    let txid = client
        .broadcast(transaction)
        .await
        .context("failed to broadcast transaction via esplora")?;

    Ok(txid)
}
