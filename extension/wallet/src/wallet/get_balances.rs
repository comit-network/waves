use crate::{
    esplora::EsploraClient,
    wallet::{current, BalanceEntry, Wallet},
};
use anyhow::Result;
use futures::lock::Mutex;

pub async fn get_balances(
    name: &str,
    current_wallet: &Mutex<Option<Wallet>>,
    client: &EsploraClient,
) -> Result<Vec<BalanceEntry>> {
    let mut wallet = current(name, current_wallet).await?;
    wallet.sync(client).await?;

    let balances = wallet.compute_balances();

    Ok(balances)
}
