use crate::{
    esplora::EsploraClient,
    wallet::{current, Wallet},
};
use anyhow::Result;
use baru::swap::alice_finalize_transaction;
use elements::{Transaction, Txid};
use futures::lock::Mutex;

pub(crate) async fn sign_and_send_swap_transaction(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    transaction: Transaction,
    client: &EsploraClient,
) -> Result<Txid, Error> {
    let mut wallet = current(&name, current_wallet)
        .await
        .map_err(Error::LoadWallet)?;
    wallet.sync(&*client).await.map_err(Error::SyncWallet)?;

    let transaction = alice_finalize_transaction(transaction, |transaction| async {
        Ok(wallet.sign(transaction))
    })
    .await
    .map_err(Error::Sign)?;

    let txid = client.broadcast(transaction).await.map_err(Error::Send)?;

    Ok(txid)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Wallet is not loaded: {0}")]
    LoadWallet(anyhow::Error),
    #[error("Failed to sign transaction: {0}")]
    Sign(anyhow::Error),
    #[error("Failed to broadcast transaction: {0}")]
    Send(anyhow::Error),
    #[error("Could not sync wallet: {0}")]
    SyncWallet(anyhow::Error),
}
