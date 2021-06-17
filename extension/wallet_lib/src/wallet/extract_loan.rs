use crate::{
    constants::{NATIVE_ASSET_ID, USDT_ASSET_ID},
    storage::Storage,
    wallet::{compute_balances, current, get_txouts, Wallet},
    TradeSide,
};
use covenants::{Borrower0, LoanResponse};
use elements::{bitcoin::util::amount::Amount, secp256k1_zkp::SECP256K1};
use futures::lock::Mutex;
use rust_decimal::Decimal;

pub async fn extract_loan(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    loan_response: String,
) -> Result<LoanDetails, Error> {
    let loan_response: LoanResponse = serde_json::from_str(&loan_response)?;

    let wallet = current(&name, current_wallet).await?;

    let txouts = get_txouts(&wallet, |utxo, txout| Ok(Some((utxo, txout)))).await?;
    let balances = compute_balances(
        &wallet,
        &txouts
            .iter()
            .map(|(_, txout)| txout)
            .cloned()
            .collect::<Vec<_>>(),
    );

    let storage = Storage::local_storage().map_err(|_| Error::LoadState)?;
    let borrower = storage
        .get_item::<String>("borrower_state")
        .map_err(|_| Error::LoadState)?
        .ok_or(Error::LoadState)?;
    let borrower: Borrower0 =
        serde_json::from_str(&borrower).map_err(|_| Error::DeserializeState)?;

    let timelock = loan_response.timelock;
    let borrower = borrower.interpret(SECP256K1, loan_response)?;

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

    // TODO: Model separate errors for `borrower_state` storage
    storage
        .set_item(
            "borrower_state",
            serde_json::to_string(&borrower).map_err(|_| Error::SaveState)?,
        )
        .map_err(|_| Error::SaveState)?;

    let loan_details = LoanDetails::new(
        borrower.collateral_amount,
        collateral_balance,
        borrower.principal_tx_out_amount,
        principal_balance,
        timelock,
    )?;

    Ok(loan_details)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LoanDetails {
    pub collateral: TradeSide,
    pub principal: TradeSide,
    pub principal_repayment: Decimal,
    // TODO: Express as target date or number of days instead?
    pub term: u64,
}

impl LoanDetails {
    fn new(
        collateral_amount: Amount,
        collateral_balance: Decimal,
        principal_amount: Amount,
        principal_balance: Decimal,
        timelock: u64,
    ) -> Result<Self, TradeSideError> {
        let collateral = TradeSide::new_sell(
            *NATIVE_ASSET_ID,
            collateral_amount.as_sat(),
            collateral_balance,
        )?;

        let principal =
            TradeSide::new_buy(*USDT_ASSET_ID, principal_amount.as_sat(), principal_balance)?;

        Ok(Self {
            collateral,
            principal_repayment: principal.amount,
            principal,
            term: timelock,
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to build trade side: {0}")]
pub struct TradeSideError(#[from] anyhow::Error);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to deserialise loan response: {0}")]
    LoanResponseDeserialization(#[from] serde_json::Error),
    #[error("Failed to load borrower state")]
    LoadState,
    #[error("Failed to save borrower state")]
    SaveState,
    #[error("Failed to deserialize borrower state")]
    DeserializeState,
    #[error("Failed to interpret loan response")]
    LoanResponseInterpretation(#[from] anyhow::Error),
    #[error("Not enough collateral to put up for loan")]
    InsufficientCollateral,
    #[error("Failed to build loan details")]
    LoanDetails(#[from] TradeSideError),
}
