use crate::{
    esplora::EsploraClient,
    wallet::{current, Wallet},
    TradeSide,
};
use anyhow::{bail, Context, Result};
use elements::Transaction;
use futures::lock::Mutex;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

// TODO: Public APIs should return specific error struct/enum
pub async fn extract_trade(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    transaction: Transaction,
    client: &EsploraClient,
) -> Result<Trade> {
    let mut wallet = current(&name, current_wallet).await?;
    wallet.sync(&*client).await?;

    let balances = wallet.compute_balances();

    let our_inputs = wallet.find_our_input_indices_in_transaction(&transaction)?;

    let (sell_asset, sell_input) = our_inputs
        .into_iter()
        .into_grouping_map()
        .fold(0, |sum, _asset, value| sum + value)
        .into_iter()
        .exactly_one()
        .context("expected single input asset type")?;

    let our_outputs = wallet.find_our_ouput_indices_in_transaction(&transaction);

    let ((buy_asset, buy_amount), change_amount) = match our_outputs.as_slice() {
        [(change_asset, change_amount), buy_output] if *change_asset == sell_asset => {
            (buy_output, change_amount)
        }
        [buy_output, (change_asset, change_amount)] if *change_asset == sell_asset => {
            (buy_output, change_amount)
        }
        &_ => bail!("no output corresponds to the sell asset"),
    };

    let sell_amount = sell_input - change_amount;

    let sell_balance = balances
        .iter()
        .find_map(|entry| {
            if entry.asset == sell_asset {
                Some(entry.value)
            } else {
                None
            }
        })
        .context("no balance for sell asset")?;

    let buy_balance = balances
        .iter()
        .find_map(|entry| {
            if entry.asset == *buy_asset {
                Some(entry.value)
            } else {
                None
            }
        })
        .unwrap_or_default();

    Ok(Trade {
        sell: TradeSide::new_sell(sell_asset, sell_amount, sell_balance)?,
        buy: TradeSide::new_buy(*buy_asset, *buy_amount, buy_balance)?,
    })
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct Trade {
    pub sell: TradeSide,
    pub buy: TradeSide,
}
