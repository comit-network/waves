use crate::{
    wallet::{current, get_txouts, Wallet, DEFAULT_SAT_PER_VBYTE},
    BTC_ASSET_ID, ESPLORA_CLIENT,
};
use anyhow::{bail, Context, Result};
use elements::{
    hashes::{hash160, Hash},
    opcodes,
    script::Builder,
    secp256k1_zkp::{rand, Message, SECP256K1},
    sighash::SigHashCache,
    Address, OutPoint, SigHashType, Transaction, TxIn, TxOut, TxOutSecrets, Txid,
};
use estimate_transaction_size::estimate_virtual_size;
use futures::lock::Mutex;
use itertools::Itertools;
use rand::thread_rng;
use std::{collections::HashMap, iter};
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

    if !address.is_blinded() {
        bail!("can only withdraw to blinded addresses")
    }

    let wallet = current(&name, current_wallet).await?;
    let blinding_key = wallet.blinding_key();

    let txouts = get_txouts(&wallet, |utxo, txout| {
        let unblinded_txout = txout.unblind(SECP256K1, blinding_key)?;
        Ok(Some((utxo, txout, unblinded_txout)))
    })
    .await?;

    let prevout_values = txouts
        .iter()
        .map(|(utxo, confidential, _)| {
            (
                OutPoint {
                    txid: utxo.txid,
                    vout: utxo.vout,
                },
                confidential.value,
            )
        })
        .collect::<HashMap<_, _>>();

    let fee_estimates = client.get_fee_estimates().await?;

    let estimated_virtual_size =
        estimate_virtual_size(prevout_values.len() as u64, txouts.len() as u64);

    let fee = (estimated_virtual_size as f32
        * fee_estimates.b_6.unwrap_or_else(|| {
            let default_fee_rate = DEFAULT_SAT_PER_VBYTE;
            log::info!(
                "fee estimate for block target '6' unavailable, falling back to default fee {}",
                default_fee_rate
            );

            default_fee_rate as f32
        })) as u64; // try to get into the next 6 blocks

    let txout_inputs = txouts
        .iter()
        .map(|(_, txout, secrets)| (txout.asset, secrets))
        .collect::<Vec<_>>();

    let txouts_grouped_by_asset = txouts
        .iter()
        .map(|(utxo, _, unblinded)| (unblinded.asset, (utxo, unblinded)))
        .into_group_map()
        .into_iter()
        .map(|(asset, txouts)| {
            // calculate the total amount we want to spend for this asset
            // if this is the native asset, subtract the fee
            let total_input = txouts.iter().map(|(_, txout)| txout.value).sum::<u64>();
            let to_spend = if asset == btc_asset_id {
                log::debug!(
                    "{} is the native asset, subtracting a fee of {} from it",
                    asset,
                    fee
                );

                total_input - fee
            } else {
                total_input
            };

            log::debug!(
                "found {} UTXOs for asset {} worth {} in total",
                txouts.len(),
                asset,
                total_input
            );

            (asset, to_spend)
        })
        .collect::<Vec<_>>();

    // build transaction from grouped txouts
    let mut transaction = match txouts_grouped_by_asset.as_slice() {
        [] => bail!("no balances in wallet"),
        [(asset, _)] if *asset != btc_asset_id => {
            bail!("cannot spend from wallet without native asset L-BTC because we cannot pay a fee",)
        }
        // handle last group separately because we need to create it is as the `last_confidential` output
        [other @ .., (last_asset, to_spend_last_txout)] => {
            // first, build all "non-last" outputs
            let other_txouts = other
                .iter()
                .map(|(asset, to_spend)| {
                    let (txout, abf, vbf) = TxOut::new_not_last_confidential(
                        &mut thread_rng(),
                        SECP256K1,
                        *to_spend,
                        address.clone(),
                        *asset,
                        txout_inputs
                            .iter()
                            .map(|(asset, secrets)| (*asset, Some(*secrets)))
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )?;

                    log::debug!(
                        "constructed non-last confidential output for asset {} with value {}",
                        asset,
                        to_spend
                    );

                    Ok((txout, asset, *to_spend, abf, vbf))
                })
                .collect::<Result<Vec<_>>>()?;

            // second, make the last one, depending on the previous ones
            let last_txout = {
                let other_outputs = other_txouts
                    .iter()
                    .map(|(_, asset, value, abf, vbf)| {
                        TxOutSecrets::new(**asset, *abf, *value, *vbf)
                    })
                    .collect::<Vec<_>>();

                let (txout, _, _) = TxOut::new_last_confidential(
                    &mut thread_rng(),
                    SECP256K1,
                    *to_spend_last_txout,
                    address,
                    *last_asset,
                    txout_inputs.as_slice(),
                    other_outputs.iter().collect::<Vec<_>>().as_ref(),
                )
                .context("failed to make confidential txout")?;

                log::debug!(
                    "constructed last confidential output for asset {} with value {}",
                    last_asset,
                    to_spend_last_txout
                );

                txout
            };

            let txins = txouts
                .into_iter()
                .map(|(utxo, _, _)| TxIn {
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
                })
                .collect::<Vec<_>>();
            let txouts = other_txouts
                .iter()
                .map(|(txout, _, _, _, _)| txout)
                .chain(iter::once(&last_txout))
                .chain(iter::once(&TxOut::new_fee(fee, btc_asset_id)))
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

            let sig = SECP256K1.sign(&Message::from(sighash), &wallet.secret_key);

            let mut serialized_signature = sig.serialize_der().to_vec();
            serialized_signature.push(SigHashType::All as u8);

            vec![
                serialized_signature,
                wallet.get_public_key().serialize().to_vec(),
            ]
        }
    }

    let txid = client
        .broadcast(transaction)
        .await
        .context("failed to broadcast transaction via esplora")?;

    Ok(txid)
}
