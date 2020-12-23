use crate::{
    esplora,
    wallet::{
        current, estimate_virtual_transaction_size, get_txouts, Wallet, DEFAULT_SAT_PER_VBYTE,
        NATIVE_ASSET_ID, NATIVE_ASSET_TICKER,
    },
    SECP,
};
use anyhow::{Context, Result};
use elements_fun::{
    hashes::{hash160, Hash},
    opcodes,
    script::Builder,
    secp256k1::{rand, Message},
    sighash::SigHashCache,
    Address, OutPoint, SigHashType, Transaction, TxIn, TxOut, Txid,
};
use futures::lock::Mutex;
use itertools::Itertools;
use rand::thread_rng;
use std::{collections::HashMap, iter};
use wasm_bindgen::JsValue;

pub async fn withdraw_everything_to(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    address: Address,
) -> Result<Txid, JsValue> {
    if !address.is_blinded() {
        return Err(JsValue::from_str("can only withdraw to blinded addresses"));
    }

    let wallet = current(&name, current_wallet).await?;
    let blinding_key = wallet.blinding_key();

    let txouts = get_txouts(&wallet, |utxo, txout| {
        Ok(match txout {
            TxOut::Confidential(confidential) => {
                let unblinded_txout = confidential.unblind(&*SECP, blinding_key)?;

                Some((utxo, confidential, unblinded_txout))
            }
            TxOut::Explicit(_) => {
                log::warn!("spending explicit txouts is unsupported");
                None
            }
            TxOut::Null(_) => None,
        })
    })
    .await?;

    let prevout_values = txouts
        .iter()
        .map(|(utxo, _, unblinded)| {
            (
                OutPoint {
                    txid: utxo.txid,
                    vout: utxo.vout,
                },
                unblinded.value,
            )
        })
        .collect::<HashMap<_, _>>();

    let fee_estimates = map_err_from_anyhow!(esplora::get_fee_estimates().await)?;

    let estimated_virtual_size =
        estimate_virtual_transaction_size(prevout_values.len() as u64, txouts.len() as u64);

    let fee = (estimated_virtual_size as f32
        * fee_estimates.b_6.unwrap_or_else(|| {
            let default_fee_rate = DEFAULT_SAT_PER_VBYTE;
            log::info!(
                "fee estimate for block target '6' unavailable, falling back to default fee {}",
                default_fee_rate
            );

            default_fee_rate
        })) as u64; // try to get into the next 6 blocks

    let txouts_grouped_by_asset = txouts
        .into_iter()
        .group_by(|(_, _, unblinded)| unblinded.asset);

    // prepare the data exactly as we need it to create the transaction
    let txouts_grouped_by_asset = (&txouts_grouped_by_asset)
        .into_iter()
        .map(|(asset, txouts)| {
            let txouts = txouts.collect::<Vec<_>>();

            // calculate the total amount we want to spend for this asset
            // if this is the native asset, subtract the fee
            let total_input = txouts.iter().map(|(_, _, txout)| txout.value).sum::<u64>();
            let to_spend = if asset == NATIVE_ASSET_ID {
                log::debug!(
                    "{} is the native asset, subtracting a fee of {} from it",
                    asset,
                    fee
                );

                total_input - fee
            } else {
                total_input
            };

            // re-arrange the data into the format needed for creating the transaction
            // this creates two vectors:
            // 1. the `TxIn`s that will go into the transaction
            // 2. the "inputs" required for constructing a blinded `TxOut`
            let (txins, txout_inputs) = txouts
                .into_iter()
                .map(|(utxo, confidential, unblinded)| {
                    (
                        TxIn {
                            previous_output: OutPoint {
                                txid: utxo.txid,
                                vout: utxo.vout,
                            },
                            is_pegin: false,
                            has_issuance: false,
                            script_sig: Default::default(),
                            sequence: 0,
                            asset_issuance: Default::default(),
                            witness: Default::default(),
                        },
                        (
                            unblinded.asset,
                            unblinded.value,
                            confidential.asset,
                            unblinded.asset_blinding_factor,
                            unblinded.value_blinding_factor,
                        ),
                    )
                })
                .unzip::<_, _, Vec<_>, Vec<_>>();

            log::debug!(
                "found {} UTXOs for asset {} worth {} in total",
                txins.len(),
                asset,
                total_input
            );

            (asset, txins, txout_inputs, to_spend)
        })
        .collect::<Vec<_>>();

    // build transaction from grouped txouts
    let mut transaction = match txouts_grouped_by_asset.as_slice() {
        [] => return Err(JsValue::from_str("no balances in wallet")),
        [(asset, _, _, _)] if asset != &NATIVE_ASSET_ID => {
            return Err(JsValue::from_str(&format!(
                "cannot spend from wallet without native asset {} because we cannot pay a fee",
                NATIVE_ASSET_TICKER
            )))
        }
        // handle last group separately because we need to create it is as the `last_confidential` output
        [other @ .., (last_asset, last_txins, last_txout_inputs, to_spend_last_txout)] => {
            // first, build all "non-last" outputs
            let other_txouts = map_err_from_anyhow!(other
                .iter()
                .map(|(asset, txins, txout_inputs, to_spend)| {
                    let (txout, abf, vbf) = TxOut::new_not_last_confidential(
                        &mut thread_rng(),
                        &*SECP,
                        *to_spend,
                        address.clone(),
                        *asset,
                        &txout_inputs,
                    )?;

                    log::debug!(
                        "constructed non-last confidential output for asset {} with value {}",
                        asset,
                        to_spend
                    );

                    Ok((txins, txout, *to_spend, abf, vbf))
                })
                .collect::<Result<Vec<_>>>())?;

            // second, make the last one, depending on the previous ones
            let last_txout = {
                let other_outputs = other_txouts
                    .iter()
                    .map(|(_, _, value, abf, vbf)| (*value, *abf, *vbf))
                    .collect::<Vec<_>>();

                let txout = map_err_from_anyhow!(TxOut::new_last_confidential(
                    &mut thread_rng(),
                    &*SECP,
                    *to_spend_last_txout,
                    address,
                    *last_asset,
                    last_txout_inputs.as_slice(),
                    other_outputs.as_slice()
                )
                .context("failed to make confidential txout"))?;

                log::debug!(
                    "constructed last confidential output for asset {} with value {}",
                    last_asset,
                    to_spend_last_txout
                );

                txout
            };

            // concatenate all inputs and outputs together to build the transaction
            let txins = other_txouts
                .iter()
                .map(|(txins, _, _, _, _)| txins.iter())
                .flatten()
                .chain(last_txins.iter())
                .cloned()
                .collect::<Vec<_>>();
            let txouts = other_txouts
                .iter()
                .map(|(_, txout, _, _, _)| txout)
                .chain(iter::once(&last_txout))
                .cloned()
                .collect::<Vec<_>>();

            Transaction {
                version: 2,
                lock_time: 0,
                input: txins,
                output: txouts,
            }
        }
    };

    let tx_clone = transaction.clone();
    let mut cache = SigHashCache::new(&tx_clone);

    for (index, input) in transaction.input.iter_mut().enumerate() {
        input.witness.script_witness = {
            let hash = hash160::Hash::hash(&wallet.get_public_key().serialize());
            let script = Builder::new()
                .push_opcode(opcodes::all::OP_DUP)
                .push_opcode(opcodes::all::OP_HASH160)
                .push_slice(&hash.into_inner())
                .push_opcode(opcodes::all::OP_EQUALVERIFY)
                .push_opcode(opcodes::all::OP_CHECKSIG)
                .into_script();

            let sighash = cache.segwitv0_sighash(
                index,
                &script,
                prevout_values[&input.previous_output],
                SigHashType::All,
            );

            let sig = SECP.sign(&Message::from(sighash), &wallet.secret_key);

            let mut serialized_signature = sig.serialize_der().to_vec();
            serialized_signature.push(SigHashType::All as u8);

            vec![
                serialized_signature,
                wallet.get_public_key().serialize().to_vec(),
            ]
        }
    }

    let txid = map_err_from_anyhow!(esplora::broadcast(transaction)
        .await
        .context("failed to broadcast transaction via esplora"))?;

    Ok(txid)
}
