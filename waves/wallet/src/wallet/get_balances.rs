use crate::{
    esplora,
    esplora::AssetDescription,
    wallet::{current, get_txouts, Wallet, NATIVE_ASSET_TICKER},
    SECP,
};
use anyhow::Result;
use elements_fun::{AssetId, TxOut};
use futures::{
    lock::Mutex,
    stream::{FuturesUnordered, StreamExt},
};
use itertools::Itertools;
use rust_decimal::Decimal;
use std::future::Future;
use wasm_bindgen::JsValue;

pub async fn get_balances(
    name: &str,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<Vec<BalanceEntry>, JsValue> {
    let wallet = current(name, current_wallet).await?;

    let txouts = get_txouts(&wallet, |_, txout| Ok(Some(txout))).await?;

    let balances = compute_balances(
        &wallet,
        &txouts,
        esplora::fetch_asset_description,
        &NATIVE_ASSET_TICKER,
    )
    .await;

    Ok(balances)
}

/// A single balance entry as returned by [`get_balances`].
#[derive(Debug, serde::Serialize)]
pub struct BalanceEntry {
    value: Decimal,
    asset: AssetId,
    /// The ticker symbol of the asset.
    ///
    /// Not all assets are part of the registry and as such, not all of them have a ticker symbol.
    ticker: Option<String>,
}

impl BalanceEntry {
    /// Construct a new [`BalanceEntry`] using the given value and [`AssetDescription`].
    ///
    /// [`AssetDescriptions`] are different for native vs user-issued assets. To properly handle these cases, we pass in the `native_asset_ticker` that should be used in case we are handling a native asset.
    pub fn for_asset(value: u64, asset: AssetDescription, native_asset_ticker: &str) -> Self {
        let precision = if asset.is_native_asset() {
            8
        } else {
            asset.precision.unwrap_or(0)
        };

        let mut decimal = Decimal::from(value);
        decimal
            .set_scale(precision)
            .expect("precision must be < 28");

        let ticker = if asset.is_native_asset() {
            Some(native_asset_ticker.to_owned())
        } else {
            asset.ticker
        };

        Self {
            value: decimal,
            asset: asset.asset_id,
            ticker,
        }
    }
}

/// A pure function to compute the balances of the wallet given a set of [`TxOut`]s.
///
/// This function needs an `asset_resolver` that can return asset descriptions for the assets included in the [`TxOut`]s.
async fn compute_balances<R, F>(
    wallet: &Wallet,
    txouts: &[TxOut],
    asset_resolver: R,
    native_asset_ticker: &str,
) -> Vec<BalanceEntry>
where
    R: Fn(AssetId) -> F + Copy,
    F: Future<Output = Result<AssetDescription>>,
{
    let grouped_txouts = txouts
        .iter()
        .filter_map(|utxo| match utxo {
            TxOut::Explicit(explicit) => Some((explicit.asset.0, explicit.value.0)),
            TxOut::Confidential(confidential) => {
                match confidential.unblind(&*SECP, wallet.blinding_key()) {
                    Ok(unblinded_txout) => Some((unblinded_txout.asset, unblinded_txout.value)),
                    Err(e) => {
                        log::warn!("failed to unblind txout: {}", e);
                        None
                    }
                }
            }
            TxOut::Null(_) => None,
        })
        .group_by(|(asset, _)| *asset);

    (&grouped_txouts)
        .into_iter()
        .map(|(asset, utxos)| async move {
            let ad = match asset_resolver(asset).await {
                Ok(ad) => ad,
                Err(e) => {
                    log::debug!("failed to fetched asset description: {}", e);

                    AssetDescription::default(asset)
                }
            };
            let total_sum = utxos.map(|(_, value)| value).sum();

            BalanceEntry::for_asset(total_sum, ad, native_asset_ticker)
        })
        .collect::<FuturesUnordered<_>>()
        .collect()
        .await
}
