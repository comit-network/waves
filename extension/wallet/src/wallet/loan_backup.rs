use elements::Txid;
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};

use crate::{
    storage::Storage,
    wallet::{current, sign_loan::update_open_loans, LoanDetails},
    Error, Wallet,
};
use baru::loan::Borrower1;

/// Represents a backup-able loan
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct BackupDetails {
    loan_details: LoanDetails,
    borrower: Borrower1,
}

pub async fn create_loan_backup(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    txid: Txid,
) -> Result<BackupDetails, Error> {
    let storage = Storage::local_storage().map_err(Error::Storage)?;

    // We get a hold of the wallet to ensure that it is loaded. This is a security mechanism
    // to ensure no unauthorized access to the data.
    // Ideally all data is encrypted but that's just how it is :)
    let _ = current(&name, current_wallet).await.unwrap();

    let open_loans = match storage
        .get_item::<String>("open_loans")
        .map_err(Error::Load)?
    {
        Some(open_loans) => serde_json::from_str(&open_loans).map_err(Error::Deserialize)?,
        None => Vec::<LoanDetails>::new(),
    };

    let loan_details = open_loans
        .iter()
        .find(|loan_details| loan_details.txid == txid)
        .ok_or(Error::LoanNotFound)?;

    // get the borrower from loan_state
    let borrower = storage
        .get_item::<String>(&format!("loan_state:{}", txid))
        .map_err(Error::Load)?
        .ok_or(Error::EmptyState)?;
    let borrower = serde_json::from_str::<Borrower1>(&borrower).map_err(Error::Deserialize)?;

    Ok(BackupDetails {
        loan_details: loan_details.clone(),
        borrower,
    })
}

pub fn load_loan_backup(backup_details: BackupDetails) -> Result<(), Error> {
    let storage = Storage::local_storage().map_err(Error::Storage)?;

    let _ = update_open_loans(
        storage,
        &backup_details.borrower,
        backup_details.loan_details,
    )?;

    Ok(())
}
