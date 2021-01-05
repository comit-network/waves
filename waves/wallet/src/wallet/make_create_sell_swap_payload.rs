use crate::{
    esplora,
    wallet::{
        coin_selection, coin_selection::coin_select, current, get_txouts, CreateSwapPayload,
        SwapUtxo, Wallet, DEFAULT_SAT_PER_VBYTE, NATIVE_ASSET_ID,
    },
    SECP,
};
use anyhow::{Context, Result};
use elements_fun::{bitcoin::Amount, transaction, OutPoint, TxOut};
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
                let unblinded_txout = confidential.unblind(&*SECP, blinding_key)?;
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

    let fee_estimates = map_err_from_anyhow!(esplora::get_fee_estimates().await)?;

    let chosen_fee_rate = fee_estimates.b_6.unwrap_or(DEFAULT_SAT_PER_VBYTE);

    let fee_for_our_output = (transaction::avg_vbytes::OUTPUT as f32 * chosen_fee_rate) as u64;
    let output = map_err_from_anyhow!(coin_select(
        utxos,
        btc,
        chosen_fee_rate,
        Amount::from_sat(fee_for_our_output)
    )
    .context("failed to select coins"))?;

    let payload = CreateSwapPayload {
        address_change: wallet.get_address()?,
        address_redeem: wallet.get_address()?,
        alice_inputs: output
            .coins
            .into_iter()
            .map(|utxo| SwapUtxo {
                outpoint: utxo.outpoint,
                blinding_key,
            })
            .collect(),
        fee: output.recommended_fee,
        btc_amount: output.target_amount,
    };

    Ok(payload)
}
