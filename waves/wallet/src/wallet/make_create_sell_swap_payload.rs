use crate::wallet::{
    coin_selection, coin_selection::coin_select, current, get_txouts, CreateSwapPayload, SwapUtxo,
    Wallet, NATIVE_ASSET_ID,
};
use anyhow::{Context, Result};
use elements_fun::{bitcoin::Amount, secp256k1::SECP256K1, transaction, OutPoint, TxOut};
use futures::lock::Mutex;
use wasm_bindgen::JsValue;

pub async fn make_create_sell_swap_payload(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    btc: Amount,
) -> Result<CreateSwapPayload, JsValue> {
    let wallet = current(&name, current_wallet).await?;
    let blinding_key = wallet.blinding_key();

    let utxos = get_txouts(&wallet, |utxo, txout| {
        Ok(match txout {
            TxOut::Confidential(confidential) => {
                let unblinded_txout = confidential.unblind(SECP256K1, blinding_key)?;
                let outpoint = OutPoint {
                    txid: utxo.txid,
                    vout: utxo.vout,
                };
                let candidate_asset = unblinded_txout.asset;

                if candidate_asset == NATIVE_ASSET_ID {
                    Some(coin_selection::Utxo {
                        outpoint,
                        value: unblinded_txout.value,
                        script_pubkey: confidential.script_pubkey,
                        asset: candidate_asset,
                    })
                } else {
                    log::debug!(
                        "utxo {} is not the sell asset {}, ignoring",
                        outpoint,
                        candidate_asset
                    );
                    None
                }
            }
            TxOut::Explicit(_) => {
                log::warn!("swapping explicit txouts is unsupported");
                None
            }
            TxOut::Null(_) => None,
        })
    })
    .await?;

    // Bob currently hardcodes a fee-rate of 1 sat / vbyte, hence there is no need for us to perform fee estimation.
    // Later on, both parties should probably agree on a block-target and use the same estimation service.
    let bobs_fee_rate = Amount::from_sat(1);
    let fee_offset = calculate_fee_offset(bobs_fee_rate);

    let output =
        map_err_from_anyhow!(
            coin_select(utxos, btc, bobs_fee_rate.as_sat() as f32, fee_offset)
                .context("failed to select coins")
        )?;

    let payload = CreateSwapPayload {
        address: wallet.get_address()?,
        alice_inputs: output
            .coins
            .into_iter()
            .map(|utxo| SwapUtxo {
                outpoint: utxo.outpoint,
                blinding_key,
            })
            .collect(),
        btc_amount: output.target_amount,
    };

    Ok(payload)
}

/// Calculate the fee offset required for the coin selection algorithm.
///
/// We are calculating this fee offset here so that we select enough coins to pay for the asset + the fee.
fn calculate_fee_offset(fee_sats_per_vbyte: Amount) -> Amount {
    let bobs_outputs = 2; // bob will create two outputs for himself (receive + change)
    let our_output = 1; // we have one additional output (the change output is priced in by the coin-selection algorithm)

    let fee_offset = ((bobs_outputs + our_output) * transaction::avg_vbytes::OUTPUT)
        * fee_sats_per_vbyte.as_sat();

    Amount::from_sat(fee_offset)
}
