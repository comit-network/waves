use crate::{
    esplora::EsploraClient,
    storage::Storage,
    wallet::{current, Wallet},
    BTC_ASSET_ID, USDT_ASSET_ID,
};
use baru::{input::Input, loan::Borrower0};
use elements::{
    bitcoin::{util::amount::Amount, PublicKey},
    Address,
};
use estimate_transaction_size::avg_vbytes;
use futures::lock::Mutex;
use rand::thread_rng;
use wasm_bindgen::UnwrapThrowExt;

pub async fn make_loan_request(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    collateral_amount: Amount,
    fee_rate: Amount,
    client: &EsploraClient,
) -> Result<LoanRequestWalletParams, Error> {
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
    wallet.sync(&*client).await.map_err(Error::SyncWallet)?;

    let fee_offset = calculate_fee_offset(fee_rate);

    let collateral_inputs = wallet
        .coin_selection(
            collateral_amount,
            btc_asset_id,
            fee_rate.as_sat() as f32,
            fee_offset,
        )
        .map_err(Error::CoinSelection)?;

    let borrower_state_0 = Borrower0::new(
        &mut thread_rng(),
        collateral_inputs,
        wallet.address(),
        wallet.blinding_secret_key(),
        collateral_amount,
        fee_rate,
        btc_asset_id,
        usdt_asset_id,
    )
    .await
    .map_err(Error::BuildBorrowerState)?;

    let storage = Storage::local_storage().map_err(Error::Storage)?;
    storage
        .set_item(
            "borrower_state",
            serde_json::to_string(&borrower_state_0).map_err(Error::Serialize)?,
        )
        .map_err(Error::Save)?;

    let loan_request = LoanRequestWalletParams::new(
        *borrower_state_0.collateral_amount(),
        borrower_state_0.collateral_inputs().to_vec(),
        borrower_state_0.fee_sats_per_vbyte(),
        borrower_state_0.pk(),
        borrower_state_0.address().clone(),
    );

    Ok(loan_request)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoanRequestWalletParams {
    #[serde(with = "::elements::bitcoin::util::amount::serde::as_sat")]
    pub collateral_amount: Amount,
    pub collateral_inputs: Vec<Input>,
    #[serde(with = "::elements::bitcoin::util::amount::serde::as_sat")]
    pub fee_sats_per_vbyte: Amount,
    pub borrower_pk: PublicKey,
    pub borrower_address: Address,
}

impl LoanRequestWalletParams {
    fn new(
        collateral_amount: Amount,
        collateral_inputs: Vec<Input>,
        fee_sats_per_vbyte: Amount,
        borrower_pk: PublicKey,
        borrower_address: Address,
    ) -> Self {
        Self {
            collateral_amount,
            collateral_inputs,
            fee_sats_per_vbyte,
            borrower_pk,
            borrower_address,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Wallet is not loaded {0}")]
    LoadWallet(anyhow::Error),
    #[error("Failed to construct borrower state: {0}")]
    BuildBorrowerState(anyhow::Error),
    #[error("Storage error: {0}")]
    Storage(anyhow::Error),
    #[error("Failed to save item to storage: {0}")]
    Save(anyhow::Error),
    #[error("Serialization failed: {0}")]
    Serialize(serde_json::Error),
    #[error("Could not sync wallet: {0}")]
    SyncWallet(anyhow::Error),
    #[error("Coin selection: {0}")]
    CoinSelection(anyhow::Error),
}

/// Calculate the fee offset required for the coin selection algorithm.
///
/// We are calculating this fee offset here so that we select enough coins to pay for the asset + the fee.
fn calculate_fee_offset(fee_sats_per_vbyte: Amount) -> Amount {
    let principal_outputs = 2; // one to pay the principal to the borrower and another as change for the lender
    let fee_offset = (principal_outputs * avg_vbytes::OUTPUT) * fee_sats_per_vbyte.as_sat();

    Amount::from_sat(fee_offset)
}
