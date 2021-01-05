use crate::wallet::{current, get_txouts, Wallet};
use anyhow::{Context, Result};
use elements_fun::{secp256k1::SECP256K1, AssetId, Transaction, TxOut, UnblindError};
use futures::lock::Mutex;
use serde::Serialize;
use wasm_bindgen::JsValue;

#[derive(Serialize)]
pub struct TransactionElements {
    our_inputs: Vec<(AssetId, u64)>,
    our_outputs: Vec<(AssetId, u64)>,
}

pub async fn decompose_transaction(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    transaction: Transaction,
) -> Result<TransactionElements, JsValue> {
    let wallet = current(&name, current_wallet).await?;
    let blinding_key = wallet.blinding_key();

    let our_inputs = get_txouts(&wallet, |utxo, ref txout| {
        transaction
            .input
            .iter()
            .find_map(|txin| {
                if utxo.txid == txin.previous_output.txid && utxo.vout == txin.previous_output.vout
                {
                    match txout {
                        TxOut::Explicit(txout) => Some(Ok((txout.asset.0, txout.value.0))),
                        TxOut::Confidential(confidential) => Some(
                            confidential
                                .unblind(SECP256K1, blinding_key)
                                .map(|unblinded| (unblinded.asset, unblinded.value)),
                        ),
                        TxOut::Null(_) => None,
                    }
                } else {
                    None
                }
            })
            .transpose()
            .context("failed to unblind one of our inputs")
    })
    .await?;

    let our_address = wallet.get_address()?;
    let our_outputs = map_err_from_anyhow!(transaction
        .output
        .iter()
        .filter_map(|txout| match txout {
            TxOut::Explicit(txout) if txout.script_pubkey == our_address.script_pubkey() =>
                Some(Ok((txout.asset.0, txout.value.0))),
            TxOut::Confidential(confidential) => {
                match confidential.unblind(SECP256K1, blinding_key) {
                    Ok(unblinded) => Some(Ok((unblinded.asset, unblinded.value))),
                    _ => None,
                }
            }
            TxOut::Explicit(_) => {
                log::debug!(
                    "ignoring explicit outputs that do not pay to our address, including fees"
                );
                None
            }
            TxOut::Null(_) => None,
        })
        .collect::<Result<Vec<_>, UnblindError>>())?;

    Ok(TransactionElements {
        our_inputs,
        our_outputs,
    })
}
