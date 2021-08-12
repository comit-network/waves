use crate::{
    esplora::EsploraClient,
    wallet::{current, CreateSwapPayload, SwapUtxo, Wallet},
    BTC_ASSET_ID, USDT_ASSET_ID,
};
use baru::avg_vbytes;
use elements::{bitcoin::Amount, AssetId};
use futures::lock::Mutex;
use wasm_bindgen::UnwrapThrowExt;

pub async fn make_buy_create_swap_payload(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    sell_amount: Amount,
    client: &EsploraClient,
) -> Result<CreateSwapPayload, Error> {
    let btc_asset_id = {
        let guard = BTC_ASSET_ID.lock().expect_throw("can get lock");
        *guard
    };
    let usdt_asset_id = {
        let guard = USDT_ASSET_ID.lock().expect_throw("can get lock");
        *guard
    };

    make_create_swap_payload(
        name,
        current_wallet,
        sell_amount,
        usdt_asset_id,
        btc_asset_id,
        client,
    )
    .await
}

pub async fn make_sell_create_swap_payload(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    sell_amount: Amount,
    client: &EsploraClient,
) -> Result<CreateSwapPayload, Error> {
    let btc_asset_id = {
        let guard = BTC_ASSET_ID.lock().expect_throw("can get lock");
        *guard
    };
    make_create_swap_payload(
        name,
        current_wallet,
        sell_amount,
        btc_asset_id,
        btc_asset_id,
        client,
    )
    .await
}

async fn make_create_swap_payload(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    sell_amount: Amount,
    sell_asset: AssetId,
    fee_asset: AssetId,
    client: &EsploraClient,
) -> Result<CreateSwapPayload, Error> {
    let (bobs_fee_rate, fee_offset) = if fee_asset == sell_asset {
        // Bob currently hardcodes a fee-rate of 1 sat / vbyte, hence
        // there is no need for us to perform fee estimation. Later
        // on, both parties should probably agree on a block-target
        // and use the same estimation service.
        let bobs_fee_rate = Amount::ONE_SAT;
        let fee_offset = calculate_fee_offset(bobs_fee_rate);

        (bobs_fee_rate.as_sat() as f32, fee_offset)
    } else {
        (0.0, Amount::ZERO)
    };

    let mut wallet = current(&name, current_wallet)
        .await
        .map_err(Error::LoadWallet)?;

    wallet.sync(&*client).await.map_err(Error::SyncWallet)?;

    let inputs = wallet
        .coin_selection(sell_amount, sell_asset, bobs_fee_rate, fee_offset)
        .map_err(Error::CoinSelection)?;

    Ok(CreateSwapPayload {
        address: wallet.get_address(),
        alice_inputs: inputs
            .into_iter()
            .map(|utxo| SwapUtxo {
                outpoint: utxo.txin,
                blinding_key: utxo.blinding_key,
            })
            .collect(),
        amount: sell_amount,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Wallet is not loaded: {0}")]
    LoadWallet(anyhow::Error),
    #[error("Coin selection: {0}")]
    CoinSelection(anyhow::Error),
    #[error("Could not sync wallet: {0}")]
    SyncWallet(anyhow::Error),
}

/// Calculate the fee offset required for the coin selection algorithm.
///
/// We are calculating this fee offset here so that we select enough coins to pay for the asset + the fee.
fn calculate_fee_offset(fee_sats_per_vbyte: Amount) -> Amount {
    let bobs_outputs = 2; // bob will create two outputs for himself (receive + change)
    let our_output = 1; // we have one additional output (the change output is priced in by the coin-selection algorithm)

    let fee_offset =
        ((bobs_outputs + our_output) * avg_vbytes::OUTPUT) * fee_sats_per_vbyte.as_sat();

    Amount::from_sat(fee_offset)
}
