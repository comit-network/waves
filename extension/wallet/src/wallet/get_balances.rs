use crate::{
    assets,
    esplora::EsploraClient,
    wallet::{current, Wallet},
};
use anyhow::Result;
use elements::AssetId;
use futures::lock::Mutex;

pub async fn get_balances(
    name: &str,
    current_wallet: &Mutex<Option<Wallet>>,
    client: &EsploraClient,
) -> Result<Vec<BalanceEntry>> {
    let mut wallet = current(name, current_wallet).await?;
    wallet.sync(client).await?;

    let balances = wallet
        .compute_balances()
        .into_iter()
        .map(|e| BalanceEntry {
            asset: e.asset,
            value: e.value,
            ticker: assets::lookup(e.asset).map(|(ticker, _)| ticker.to_owned()),
        })
        .collect();

    Ok(balances)
}

/// A single balance entry as returned by [`get_balances`].
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BalanceEntry {
    pub asset: AssetId,
    pub value: u64,
    pub ticker: Option<String>,
}
