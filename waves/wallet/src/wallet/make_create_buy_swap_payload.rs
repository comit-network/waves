use crate::{
    constants::USDT_ASSET_ID,
    wallet::{
        coin_selection, coin_selection::coin_select, current, get_txouts, CreateSwapPayload,
        SwapUtxo, Wallet,
    },
};
use anyhow::{Context, Result};
use bdk::bitcoin::{Amount, Denomination};
use elements::{secp256k1::SECP256K1, OutPoint};
use futures::lock::Mutex;

pub async fn make_create_buy_swap_payload(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    usdt: String,
) -> Result<CreateSwapPayload> {
    // TODO: Extract module `bobtimus::amounts` into a shared library
    // so that we can model L-USDt properly here
    let usdt = Amount::from_str_in(&usdt, Denomination::Bitcoin)
        .context("failed to parse amount from string")?;

    let wallet = current(&name, current_wallet).await?;
    let blinding_key = wallet.blinding_key();

    let utxos = get_txouts(&wallet, |utxo, txout| {
        Ok(match txout.into_confidential() {
            Some(confidential) => {
                let unblinded_txout = confidential.unblind(SECP256K1, blinding_key)?;
                let outpoint = OutPoint {
                    txid: utxo.txid,
                    vout: utxo.vout,
                };
                let candidate_asset = unblinded_txout.asset;

                if candidate_asset == *USDT_ASSET_ID {
                    Some(coin_selection::Utxo {
                        outpoint,
                        value: unblinded_txout.value,
                        script_pubkey: confidential.script_pubkey,
                        asset: candidate_asset,
                    })
                } else {
                    log::debug!(
                        "utxo {} with asset id {} is not the sell asset, ignoring",
                        outpoint,
                        candidate_asset
                    );
                    None
                }
            }
            None => {
                log::warn!("swapping explicit txouts is unsupported");
                None
            }
        })
    })
    .await?;

    log::debug!("{:?}", utxos);

    let output = coin_select(utxos, usdt, 0.0, Amount::ZERO).context("failed to select coins")?;

    log::debug!("coin select output: {:?}", output);

    Ok(CreateSwapPayload {
        address: wallet.get_address(),
        alice_inputs: output
            .coins
            .into_iter()
            .map(|utxo| SwapUtxo {
                outpoint: utxo.outpoint,
                blinding_key,
            })
            .collect(),
        amount: output.target_amount,
    })
}
