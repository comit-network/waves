use crate::wallet::{compute_balances, current, get_txouts, BalanceEntry, Wallet};
use anyhow::Result;
use futures::lock::Mutex;

pub async fn get_balances(
    name: &str,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<Vec<BalanceEntry>> {
    let wallet = current(name, current_wallet).await?;
    log::debug!("Got current wallet: {:?}", wallet);

    let txouts = get_txouts(&wallet, |_, txout| Ok(Some(txout))).await?;
    log::debug!("Got txouts: {:?}", txouts);

    let balances = compute_balances(&wallet, &txouts);

    Ok(balances)
}
