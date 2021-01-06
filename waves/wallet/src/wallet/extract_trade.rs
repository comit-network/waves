use crate::{
    assets,
    wallet::{compute_balances, current, get_txouts, Wallet},
};
use anyhow::{anyhow, Context, Result};
use elements_fun::{secp256k1::SECP256K1, AssetId, Transaction, TxOut};
use futures::lock::Mutex;
use itertools::Itertools;
use rust_decimal::Decimal;
use serde::Serialize;
use std::ops::{Add, Sub};
use wasm_bindgen::JsValue;

pub async fn extract_trade(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    transaction: Transaction,
) -> Result<Trade, JsValue> {
    let wallet = current(&name, current_wallet).await?;

    let txouts = get_txouts(&wallet, |utxo, txout| Ok(Some((utxo, txout)))).await?;
    let balances = compute_balances(
        &wallet,
        &txouts
            .iter()
            .map(|(_, txout)| txout)
            .cloned()
            .collect::<Vec<_>>(),
    );

    let blinding_key = wallet.blinding_key();

    let our_inputs = transaction
        .input
        .iter()
        .filter_map(|txin| {
            txouts
                .iter()
                .map(|(utxo, txout)| {
                    let is_ours = utxo.txid == txin.previous_output.txid
                        && utxo.vout == txin.previous_output.vout;
                    if !is_ours {
                        return Ok(None);
                    }

                    Ok(match txout {
                        TxOut::Explicit(txout) => Some((txout.asset.0, txout.value.0)),
                        TxOut::Confidential(confidential) => {
                            let unblinded = confidential.unblind(SECP256K1, blinding_key)?;

                            Some((unblinded.asset, unblinded.value))
                        }
                        TxOut::Null(_) => None,
                    })
                })
                .find_map(|res| res.transpose())
        })
        .collect::<Result<Vec<_>>>()
        .context("failed to unblind one of our inputs");
    let our_inputs = map_err_from_anyhow!(our_inputs)?;

    let sell = our_inputs
        .into_iter()
        .into_grouping_map()
        .fold(0, |sum, _asset, value| sum + value)
        .into_iter()
        .exactly_one();
    let (sell_asset, sell_input) =
        map_err_from_anyhow!(sell.context("expected single input asset type"))?;

    let our_address = wallet.get_address()?;
    let our_outputs: Option<(_, _)> = transaction
        .output
        .iter()
        .filter_map(|txout| match txout {
            TxOut::Explicit(txout) if txout.script_pubkey == our_address.script_pubkey() => {
                Some((txout.asset.0, txout.value.0))
            }
            TxOut::Confidential(confidential) => {
                match confidential.unblind(SECP256K1, blinding_key) {
                    Ok(unblinded) => Some((unblinded.asset, unblinded.value)),
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
        .into_grouping_map()
        .fold(0, |sum, _asset, value| sum + value)
        .into_iter()
        .collect_tuple();
    let our_outputs =
        map_err_from_anyhow!(our_outputs.context("wrong number of outputs, expected 2"))?;

    let ((buy_asset, buy_amount), change_amount) = match our_outputs {
        ((change_asset, change_amount), buy_output) if change_asset == sell_asset => {
            (buy_output, change_amount)
        }
        (buy_output, (change_asset, change_amount)) if change_asset == sell_asset => {
            (buy_output, change_amount)
        }
        _ => return map_err_from_anyhow!(Err(anyhow!("no output corresponds to the sell asset"))),
    };
    let sell_amount = sell_input - change_amount;

    let sell_balance = map_err_from_anyhow!(balances
        .iter()
        .find_map(|entry| if entry.asset == sell_asset {
            Some(entry.value)
        } else {
            None
        })
        .context("no balance for sell asset"))?;

    let buy_balance = balances
        .iter()
        .find_map(|entry| {
            if entry.asset == buy_asset {
                Some(entry.value)
            } else {
                None
            }
        })
        .unwrap_or_default();

    Ok(Trade {
        sell: map_err_from_anyhow!(TradeSide::new_sell(sell_asset, sell_amount, sell_balance))?,
        buy: map_err_from_anyhow!(TradeSide::new_buy(buy_asset, buy_amount, buy_balance))?,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSide {
    ticker: String,
    amount: Decimal,
    balance_before: Decimal,
    balance_after: Decimal,
}

#[derive(Serialize)]
pub struct Trade {
    sell: TradeSide,
    buy: TradeSide,
}

impl TradeSide {
    fn new_sell(asset: AssetId, amount: u64, current_balance: Decimal) -> Result<Self> {
        Self::new(asset, amount, current_balance, Decimal::sub)
    }

    fn new_buy(asset: AssetId, amount: u64, current_balance: Decimal) -> Result<Self> {
        Self::new(asset, amount, current_balance, Decimal::add)
    }

    fn new(
        asset: AssetId,
        amount: u64,
        current_balance: Decimal,
        balance_after: impl Fn(Decimal, Decimal) -> Decimal,
    ) -> Result<Self> {
        let (ticker, precision) = assets::lookup(asset).context("asset not found")?;

        let mut amount = Decimal::from(amount);
        amount
            .set_scale(precision as u32)
            .expect("precision must be < 28");

        Ok(Self {
            ticker: ticker.to_owned(),
            amount,
            balance_before: current_balance,
            balance_after: balance_after(current_balance, amount),
        })
    }
}
