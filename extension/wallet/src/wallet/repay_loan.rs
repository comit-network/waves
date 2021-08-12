use baru::loan::Borrower1;
use elements::{
    bitcoin::{secp256k1::SECP256K1, util::amount::Amount},
    Txid,
};
use futures::lock::Mutex;
use rand::thread_rng;

use crate::{
    esplora::EsploraClient,
    storage::Storage,
    wallet::{current, LoanDetails},
    Wallet, DEFAULT_SAT_PER_VBYTE,
};

// TODO: Parts of the implementation are very similar to what we do in
// `sign_and_send_swap_transaction`. We could extract common
// functionality into crate-local functions
pub async fn repay_loan(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    loan_txid: Txid,
    client: &EsploraClient,
) -> Result<Txid, Error> {
    // TODO: Only abort early if this fails because the transaction
    // hasn't been mined
    if client.fetch_transaction(loan_txid).await.is_err() {
        return Err(Error::NoLoan);
    }

    let storage = Storage::local_storage().map_err(Error::Storage)?;

    let borrower = storage
        .get_item::<String>(&format!("loan_state:{}", loan_txid))
        .map_err(Error::Load)?
        .ok_or(Error::EmptyState)?;
    let borrower = serde_json::from_str::<Borrower1>(&borrower).map_err(Error::Deserialize)?;

    // We are selecting coins with an asset which cannot be
    // used to pay for fees
    let zero_fee_rate = 0f32;
    let zero_fee_offset = Amount::ZERO;
    let coin_selector = {
        let name = name.clone();
        |amount, asset| async move {
            let mut wallet = current(&name, current_wallet)
                .await
                .map_err(Error::LoadWallet)?;
            wallet.sync(client).await.map_err(Error::SyncWallet)?;
            wallet.coin_selection(amount, asset, zero_fee_rate, zero_fee_offset)
        }
    };

    let signer = |transaction| async {
        let mut wallet = current(&name, current_wallet)
            .await
            .map_err(Error::LoadWallet)?;
        wallet.sync(client).await.map_err(Error::SyncWallet)?;
        Ok(wallet.sign(transaction))
    };

    let loan_repayment_tx = borrower
        .loan_repayment_transaction(
            &mut thread_rng(),
            SECP256K1,
            coin_selector,
            signer,
            Amount::from_sat(DEFAULT_SAT_PER_VBYTE),
        )
        .await
        .map_err(Error::BuildTransaction)?;

    let repayment_txid = client
        .broadcast(loan_repayment_tx)
        .await
        .map_err(Error::SendTransaction)?;

    // TODO: Make sure that we can safely forget this i.e. sufficient
    // confirmations
    storage
        .remove_item(&format!("loan_state:{}", loan_txid))
        .map_err(Error::Delete)?;

    let open_loans = match storage
        .get_item::<String>("open_loans")
        .map_err(Error::Load)?
    {
        Some(open_loans) => serde_json::from_str(&open_loans).map_err(Error::Deserialize)?,
        None => Vec::<LoanDetails>::new(),
    };

    let open_loans = open_loans
        .iter()
        .filter(|details| loan_txid != details.txid)
        .collect::<Vec<_>>();
    storage
        .set_item(
            "open_loans",
            serde_json::to_string(&open_loans).map_err(Error::Serialize)?,
        )
        .map_err(Error::Save)?;

    Ok(repayment_txid)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Loan transaction not found in the blockchain")]
    NoLoan,
    #[error("Storage error: {0}")]
    Storage(anyhow::Error),
    #[error("Failed to load item from storage: {0}")]
    Load(anyhow::Error),
    #[error("Deserialization failed: {0}")]
    Deserialize(serde_json::Error),
    #[error("Serialization failed: {0}")]
    Serialize(serde_json::Error),
    #[error("Failed to delete item from storage: {0}")]
    Delete(anyhow::Error),
    #[error("Failed to save item to storage: {0}")]
    Save(anyhow::Error),
    #[error("Loaded empty loan state")]
    EmptyState,
    #[error("Wallet is not loaded: {0}")]
    LoadWallet(anyhow::Error),
    #[error("Failed to construct loan repayment transaction: {0}")]
    BuildTransaction(anyhow::Error),
    #[error("Failed to broadcast transaction: {0}")]
    SendTransaction(anyhow::Error),
    #[error("Could not sync wallet: {0}")]
    SyncWallet(anyhow::Error),
}
