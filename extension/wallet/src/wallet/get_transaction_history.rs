use anyhow::Result;
use elements::Txid;
use futures::lock::Mutex;

use crate::{esplora, wallet::current, Wallet};

pub async fn get_transaction_history(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<Vec<Txid>> {
    let wallet = current(&name, current_wallet).await?;

    // We have a single address, so looking for the transaction
    // history of said address is sufficient
    let address = wallet.get_address();
    let history = esplora::fetch_transaction_history(&address).await?;

    Ok(history)
}
