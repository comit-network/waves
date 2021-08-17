use crate::{
    assets::{self},
    DEFAULT_SAT_PER_VBYTE,
};
use anyhow::{bail, Context, Result};
use elements::{
    bitcoin::{secp256k1::SecretKey, util::amount::Amount},
    Address, AssetId, OutPoint, Txid,
};
use futures::lock::{MappedMutexGuard, Mutex, MutexGuard};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    fmt,
    ops::{Add, Sub},
    str,
};

pub use baru::Wallet;
pub use create_new::{bip39_seed_words, create_from_bip39};
pub use extract_loan::{extract_loan, Error as ExtractLoanError};
pub use extract_trade::{extract_trade, Trade};
pub use get_address::get_address;
pub use get_balances::get_balances;
pub use get_status::{get_status, WalletStatus};
pub use get_transaction_history::get_transaction_history;
pub use load_existing::load_existing;
pub use loan_backup::{create_loan_backup, load_loan_backup, BackupDetails};
pub use make_create_swap_payload::{
    make_buy_create_swap_payload, make_sell_create_swap_payload, Error as MakePayloadError,
};
pub use make_loan_request::{make_loan_request, Error as MakeLoanRequestError};
pub use repay_loan::{repay_loan, Error as RepayLoanError};
pub(crate) use sign_and_send_swap_transaction::sign_and_send_swap_transaction;
pub(crate) use sign_loan::sign_loan;
pub use unload_current::unload_current;
pub use withdraw_everything_to::withdraw_everything_to;

mod create_new;
mod extract_loan;
mod extract_trade;
mod get_address;
mod get_balances;
mod get_status;
mod get_transaction_history;
mod load_existing;
mod loan_backup;
mod make_create_swap_payload;
mod make_loan_request;
mod repay_loan;
mod sign_and_send_swap_transaction;
mod sign_loan;
mod unload_current;
mod withdraw_everything_to;

async fn current<'n, 'w>(
    name: &'n str,
    current_wallet: &'w Mutex<Option<Wallet>>,
) -> Result<MappedMutexGuard<'w, Option<Wallet>, Wallet>> {
    let mut guard = current_wallet.lock().await;

    match &mut *guard {
        Some(wallet) if wallet.name() == name => {}
        _ => bail!("wallet with name '{}' is currently not loaded", name),
    };

    Ok(MutexGuard::map(guard, |w| w.as_mut().unwrap()))
}

#[derive(Default)]
pub struct ListOfWallets(Vec<String>);

impl ListOfWallets {
    fn has(&self, wallet: &str) -> bool {
        self.0.iter().any(|w| w == wallet)
    }

    fn add(&mut self, wallet: String) {
        self.0.push(wallet);
    }
}

impl str::FromStr for ListOfWallets {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split('\t');

        Ok(ListOfWallets(split.map(|s| s.to_owned()).collect()))
    }
}

impl fmt::Display for ListOfWallets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("\t"))
    }
}

/// Represents the payload for creating a swap.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateSwapPayload {
    pub alice_inputs: Vec<SwapUtxo>,
    pub address: Address,
    #[serde(with = "elements::bitcoin::util::amount::serde::as_sat")]
    pub amount: elements::bitcoin::Amount,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct SwapUtxo {
    pub outpoint: OutPoint,
    pub blinding_key: SecretKey,
}

pub use baru::BalanceEntry;

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TradeSide {
    pub ticker: String,
    pub amount: Decimal,
    pub balance_before: Decimal,
    pub balance_after: Decimal,
}

impl TradeSide {
    fn new_sell(asset: AssetId, amount: u64, current_balance: u64) -> Result<Self> {
        Self::new(asset, amount, current_balance, Decimal::sub)
    }

    fn new_buy(asset: AssetId, amount: u64, current_balance: u64) -> Result<Self> {
        Self::new(asset, amount, current_balance, Decimal::add)
    }

    fn new(
        asset: AssetId,
        amount: u64,
        current_balance: u64,
        balance_after: impl Fn(Decimal, Decimal) -> Decimal,
    ) -> Result<Self> {
        let (ticker, precision) = assets::lookup(asset).context("asset not found")?;

        let mut amount = Decimal::from(amount);
        amount
            .set_scale(precision as u32)
            .expect("precision must be < 28");

        let current_balance = Decimal::from(current_balance);
        amount
            .set_scale(precision as u32)
            .expect("precision must be < 28");

        Ok(Self {
            ticker: ticker.to_owned(),
            amount,
            balance_before: current_balance,
            balance_after: balance_after(current_balance, amount),
        })
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoanDetails {
    pub collateral: TradeSide,
    pub principal: TradeSide,
    // TODO: This should be a u64 (sats) to prevent loss of precision when converting to a double in
    // javascript land
    pub principal_repayment: Decimal,
    // TODO: Express as target date or number of days instead?
    pub term: u32,
    pub txid: Txid,
}

impl LoanDetails {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        collateral_asset: AssetId,
        collateral_amount: Amount,
        collateral_balance: u64,
        principal_asset: AssetId,
        principal_amount: Amount,
        principal_balance: u64,
        timelock: u32,
        txid: Txid,
    ) -> Result<Self> {
        let collateral = TradeSide::new_sell(
            collateral_asset,
            collateral_amount.as_sat(),
            collateral_balance,
        )?;

        let principal = TradeSide::new_buy(
            principal_asset,
            principal_amount.as_sat(),
            principal_balance,
        )?;

        Ok(Self {
            collateral,
            principal_repayment: principal.amount,
            principal,
            term: timelock,
            txid,
        })
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod browser_tests {
    use wasm_bindgen_test::*;

    use super::*;
    use bip32::{Language, Mnemonic};

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    async fn create_new(
        name: String,
        password: String,
        current_wallet: &Mutex<Option<Wallet>>,
    ) -> Result<()> {
        let mnemonic = Mnemonic::new("globe favorite camp draw action kid soul junk space soda genre vague name brisk female circle equal fix decade gloom elbow address genius noodle", Language::English).unwrap();
        create_from_bip39(
            name,
            mnemonic,
            password,
            "elements".to_string(),
            current_wallet,
        )
        .await
    }

    fn set_elements_chain_in_local_storage() {
        crate::Storage::local_storage()
            .unwrap()
            .set_item("CHAIN", "ELEMENTS")
            .unwrap();
    }

    #[wasm_bindgen_test]
    pub async fn given_no_wallet_when_getting_address_then_fails() {
        set_elements_chain_in_local_storage();

        let current_wallet = Mutex::default();
        let error = get_address("no-existent-wallet".to_owned(), &current_wallet)
            .await
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "wallet with name 'no-existent-wallet' is currently not loaded"
        );
    }

    #[wasm_bindgen_test]
    pub async fn given_a_wallet_can_get_an_address() {
        set_elements_chain_in_local_storage();

        let current_wallet = Mutex::default();
        create_new("wallet-1".to_owned(), "foo".to_owned(), &current_wallet)
            .await
            .unwrap();

        let result = get_address("wallet-1".to_owned(), &current_wallet).await;

        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    pub async fn given_a_wallet_when_unloaded_cannot_get_address() {
        let current_wallet = Mutex::default();
        create_new("wallet-2".to_owned(), "foo".to_owned(), &current_wallet)
            .await
            .unwrap();

        unload_current(&current_wallet).await;
        let error = get_address("wallet-2".to_owned(), &current_wallet)
            .await
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "wallet with name 'wallet-2' is currently not loaded"
        );
    }

    #[wasm_bindgen_test]
    pub async fn cannot_create_two_wallets_with_same_name() {
        let current_wallet = Mutex::default();

        create_new("wallet-3".to_owned(), "foo".to_owned(), &current_wallet)
            .await
            .unwrap();
        let error = create_new("wallet-3".to_owned(), "foo".to_owned(), &current_wallet)
            .await
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "wallet with name 'wallet-3' already exists"
        );
    }

    #[wasm_bindgen_test]
    pub async fn cannot_load_multiple_wallets_at_the_same_time() {
        let current_wallet = Mutex::default();

        create_new("wallet-4".to_owned(), "foo".to_owned(), &current_wallet)
            .await
            .unwrap();
        create_new("wallet-5".to_owned(), "foo".to_owned(), &current_wallet)
            .await
            .unwrap();

        let error = load_existing(
            "wallet-4".to_owned(),
            "foo".to_owned(),
            "elements".to_string(),
            &current_wallet,
        )
        .await
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "cannot load wallet 'wallet-4' because wallet 'wallet-5' is currently loaded"
        );
    }

    #[wasm_bindgen_test]
    pub async fn cannot_load_wallet_with_wrong_password() {
        let current_wallet = Mutex::default();

        create_new("wallet-6".to_owned(), "foo".to_owned(), &current_wallet)
            .await
            .unwrap();
        unload_current(&current_wallet).await;

        let error = load_existing(
            "wallet-6".to_owned(),
            "bar".to_owned(),
            "elements".to_string(),
            &current_wallet,
        )
        .await
        .unwrap_err();

        assert_eq!(error.to_string(), "bad password for wallet 'wallet-6'");
    }

    #[wasm_bindgen_test]
    pub async fn cannot_load_wallet_that_doesnt_exist() {
        let current_wallet = Mutex::default();

        let error = load_existing(
            "foobar".to_owned(),
            "bar".to_owned(),
            "elements".to_string(),
            &current_wallet,
        )
        .await
        .unwrap_err();

        assert_eq!(error.to_string(), "wallet 'foobar' does not exist");
    }

    #[wasm_bindgen_test]
    pub async fn new_wallet_is_automatically_loaded() {
        let current_wallet = Mutex::default();

        create_new("wallet-7".to_owned(), "foo".to_owned(), &current_wallet)
            .await
            .unwrap();
        let status = get_status("wallet-7".to_owned(), &current_wallet)
            .await
            .unwrap();

        assert_eq!(status.loaded, true);
    }

    #[wasm_bindgen_test]
    pub async fn given_unknown_wallet_status_returns_that_it_doesnt_exist() {
        let current_wallet = Mutex::default();

        let status = get_status("wallet-8".to_owned(), &current_wallet)
            .await
            .unwrap();

        assert_eq!(status.exists, false);
    }

    #[wasm_bindgen_test]
    pub async fn secret_key_can_be_successfully_decrypted() {
        let current_wallet = Mutex::default();

        create_new("wallet-9".to_owned(), "foo".to_owned(), &current_wallet)
            .await
            .unwrap();
        let initial_sk = {
            let guard = current_wallet.lock().await;
            let wallet = guard.as_ref().unwrap();

            wallet.secret_key()
        };

        unload_current(&current_wallet).await;

        load_existing(
            "wallet-9".to_owned(),
            "foo".to_owned(),
            "elements".to_string(),
            &current_wallet,
        )
        .await
        .unwrap();
        let loaded_sk = {
            let guard = current_wallet.lock().await;
            let wallet = guard.as_ref().unwrap();

            wallet.secret_key()
        };

        assert_eq!(initial_sk, loaded_sk);
    }
}
