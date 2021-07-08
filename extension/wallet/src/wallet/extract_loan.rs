use crate::{
    constants::{NATIVE_ASSET_ID, USDT_ASSET_ID},
    storage::Storage,
    wallet::{compute_balances, current, get_txouts, Wallet},
    LoanDetails,
};
use covenants::{Borrower0, LoanResponse};
use elements::secp256k1_zkp::SECP256K1;
use futures::lock::Mutex;

pub async fn extract_loan(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    loan_response: LoanResponse,
) -> Result<LoanDetails, Error> {
    let wallet = current(&name, current_wallet)
        .await
        .map_err(Error::LoadWallet)?;

    let txouts = get_txouts(&wallet, |utxo, txout| Ok(Some((utxo, txout))))
        .await
        .map_err(Error::GetTxOuts)?;
    let balances = compute_balances(
        &wallet,
        &txouts
            .iter()
            .map(|(_, txout)| txout)
            .cloned()
            .collect::<Vec<_>>(),
    );

    let storage = Storage::local_storage().map_err(Error::Storage)?;
    let borrower = storage
        .get_item::<String>("borrower_state")
        .map_err(Error::Load)?
        .ok_or(Error::EmptyState)?;
    let borrower = serde_json::from_str::<Borrower0>(&borrower).map_err(Error::Deserialize)?;

    let timelock = loan_response.timelock;
    let borrower = borrower
        .interpret(SECP256K1, loan_response)
        .map_err(Error::InterpretLoanResponse)?;

    let collateral_balance = balances
        .iter()
        .find_map(|entry| {
            if entry.asset == *NATIVE_ASSET_ID {
                Some(entry.value)
            } else {
                None
            }
        })
        .ok_or(Error::InsufficientCollateral)?;

    let principal_balance = balances
        .iter()
        .find_map(|entry| {
            if entry.asset == *USDT_ASSET_ID {
                Some(entry.value)
            } else {
                None
            }
        })
        .unwrap_or_default();

    let loan_txid = borrower.loan_transaction.txid();
    let loan_details = LoanDetails::new(
        borrower.collateral_amount,
        collateral_balance,
        borrower.principal_tx_out_amount,
        principal_balance,
        timelock,
        loan_txid,
    )
    .map_err(Error::LoanDetails)?;

    storage
        .set_item(
            "borrower_state",
            serde_json::to_string(&(borrower, loan_details.clone())).map_err(Error::Serialize)?,
        )
        .map_err(Error::Save)?;

    Ok(loan_details)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to deserialise loan response: {0}")]
    LoanResponseDeserialization(#[from] serde_json::Error),
    #[error("Wallet is not loaded: {0}")]
    LoadWallet(anyhow::Error),
    #[error("Failed to get transaction outputs: {0}")]
    GetTxOuts(anyhow::Error),
    #[error("Storage error: {0}")]
    Storage(anyhow::Error),
    #[error("Failed to load item from storage: {0}")]
    Load(anyhow::Error),
    #[error("Failed to save item to storage: {0}")]
    Save(anyhow::Error),
    #[error("Loaded empty borrower state")]
    EmptyState,
    #[error("Deserialization failed: {0}")]
    Deserialize(serde_json::Error),
    #[error("Serialization failed: {0}")]
    Serialize(serde_json::Error),
    #[error("Failed to interpret loan response: {0}")]
    InterpretLoanResponse(anyhow::Error),
    #[error("Not enough collateral to put up for loan")]
    InsufficientCollateral,
    #[error("Failed to build loan details: {0}")]
    LoanDetails(anyhow::Error),
}
