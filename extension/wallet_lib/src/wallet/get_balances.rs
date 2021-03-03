use crate::wallet::{compute_balances, current, get_txouts, BalanceEntry, Wallet};
use anyhow::Result;
use futures::lock::Mutex;

pub async fn get_balances(
    name: &str,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<Vec<BalanceEntry>> {
    let wallet = current(name, current_wallet).await?;

    let txouts = get_txouts(&wallet, |_, txout| Ok(Some(txout))).await?;

    let balances = compute_balances(&wallet, &txouts);

    Ok(balances)
}
