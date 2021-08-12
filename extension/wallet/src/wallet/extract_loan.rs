use crate::{
    esplora::EsploraClient,
    storage::Storage,
    wallet::{current, Wallet},
    LoanDetails, BTC_ASSET_ID, USDT_ASSET_ID,
};
use baru::loan::{Borrower0, LoanResponse};
use elements::secp256k1_zkp::SECP256K1;
use futures::lock::Mutex;
use wasm_bindgen::UnwrapThrowExt;

pub async fn extract_loan(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    loan_response: LoanResponse,
    client: &EsploraClient,
) -> Result<LoanDetails, Error> {
    let btc_asset_id = {
        let guard = BTC_ASSET_ID.lock().expect_throw("can get lock");
        *guard
    };
    let usdt_asset_id = {
        let guard = USDT_ASSET_ID.lock().expect_throw("can get lock");
        *guard
    };

    let mut wallet = current(&name, current_wallet)
        .await
        .map_err(Error::LoadWallet)?;
    wallet.sync(client).await.map_err(Error::SyncWallet)?;

    let balances = wallet.compute_balances();

    let storage = Storage::local_storage().map_err(Error::Storage)?;
    let borrower = storage
        .get_item::<String>("borrower_state")
        .map_err(Error::Load)?
        .ok_or(Error::EmptyState)?;
    let borrower = serde_json::from_str::<Borrower0>(&borrower).map_err(Error::Deserialize)?;

    let borrower = borrower
        .interpret(SECP256K1, loan_response)
        .map_err(Error::InterpretLoanResponse)?;
    let timelock = borrower.collateral_contract().timelock();

    let collateral_balance = balances
        .iter()
        .find_map(|entry| {
            if entry.asset == btc_asset_id {
                Some(entry.value)
            } else {
                None
            }
        })
        .ok_or(Error::InsufficientCollateral)?;

    let principal_balance = balances
        .iter()
        .find_map(|entry| {
            if entry.asset == usdt_asset_id {
                Some(entry.value)
            } else {
                None
            }
        })
        .unwrap_or_default();

    let loan_txid = borrower.loan_transaction().txid();
    let loan_details = LoanDetails::new(
        btc_asset_id,
        borrower.collateral_amount(),
        collateral_balance,
        usdt_asset_id,
        borrower.principal_amount(),
        principal_balance,
        *timelock,
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
    #[error("Could not sync wallet: {0}")]
    SyncWallet(anyhow::Error),
}
