use anyhow::Result;
use conquer_once::Lazy;
use covenants::LoanRequest;
use elements::{Address, Transaction, Txid};
use futures::lock::Mutex;

#[macro_use]
mod macros;

mod assets;
mod cache_storage;
mod esplora;
mod logger;
mod storage;
mod wallet;

pub use crate::wallet::*;

mod constants {
    include!(concat!(env!("OUT_DIR"), "/", "constants.rs"));
}

static LOADED_WALLET: Lazy<Mutex<Option<Wallet>>> = Lazy::new(Mutex::default);

pub fn setup() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let _ = logger::try_init();

    log::info!("wallet initialized");
}

/// Create a new wallet with the given name and password.
///
/// Fails if a wallet with this name already exists.
/// The created wallet will be automatically loaded.
pub async fn create_new_wallet(name: String, password: String) -> Result<()> {
    wallet::create_new(name, password, &LOADED_WALLET).await
}

/// Load an existing wallet.
///
/// Fails if:
///
/// - the wallet does not exist
/// - the password is wrong
pub async fn load_existing_wallet(name: String, password: String) -> Result<()> {
    wallet::load_existing(name, password, &LOADED_WALLET).await
}

/// Unload the currently loaded wallet.
///
/// Does nothing if currently no wallet is loaded.
pub async fn unload_current_wallet() {
    wallet::unload_current(&LOADED_WALLET).await
}

/// Retrieve the status of the wallet with the given name.
pub async fn wallet_status(name: String) -> Result<WalletStatus> {
    let status = wallet::get_status(name, &LOADED_WALLET).await?;

    Ok(status)
}

/// Get an address for the wallet with the given name.
///
/// Fails if the wallet is currently not loaded.
pub async fn get_address(name: String) -> Result<Address> {
    let address = wallet::get_address(name, &LOADED_WALLET).await?;

    Ok(address)
}

/// Get the balances of the currently loaded wallet.
///
/// Returns an array of [`BalanceEntry`]s.
///
/// Fails if the wallet is currently not loaded or we cannot reach the block explorer for some reason.
pub async fn get_balances(name: String) -> Result<Vec<BalanceEntry>> {
    let balance_entries = wallet::get_balances(&name, &LOADED_WALLET).await?;

    Ok(balance_entries)
}

/// Withdraw all funds to the given address.
///
/// Returns the transaction ID of the transaction that was broadcasted.
pub async fn withdraw_everything_to(name: String, address: String) -> Result<Txid> {
    let txid = wallet::withdraw_everything_to(name, &LOADED_WALLET, address).await?;

    Ok(txid)
}

/// Constructs a new [`CreateSwapPayload`] with the given USDt amount.
///
/// This will select UTXOs from the wallet to cover the given amount.
pub async fn make_buy_create_swap_payload(
    wallet_name: String,
    usdt: String,
) -> Result<CreateSwapPayload, MakePayloadError> {
    wallet::make_buy_create_swap_payload(wallet_name, &LOADED_WALLET, usdt).await
}

/// Constructs a new [`CreateSwapPayload`] with the given Bitcoin amount.
///
/// This will select UTXOs from the wallet to cover the given amount.
pub async fn make_sell_create_swap_payload(
    wallet_name: String,
    btc: String,
) -> Result<CreateSwapPayload, MakePayloadError> {
    wallet::make_sell_create_swap_payload(wallet_name, &LOADED_WALLET, btc).await
}

/// Constructs a new [`CreateSwapPayload`] with the given Bitcoin amount.
///
/// This will select UTXOs from the wallet to cover the given amount.
///
/// Additionally, sets the state of the loan protocol so that we can
/// continue after the lender sends back a response to our loan
/// request.
pub async fn make_loan_request(
    wallet_name: String,
    collateral: String,
) -> Result<LoanRequest, MakeLoanRequestError> {
    wallet::make_loan_request(wallet_name, &LOADED_WALLET, collateral).await
}

/// Sign a loan transaction in the wallet's state, if the state of the
/// current loan protocol allows it.
///
/// Returns the signed transaction.
pub async fn sign_loan(wallet_name: String) -> Result<Transaction, SignLoanError> {
    wallet::sign_loan(wallet_name, &LOADED_WALLET).await
}

/// Sign the given swap transaction and broadcast it to the network.
///
/// Returns the transaction ID.
pub async fn sign_and_send_swap_transaction(
    wallet_name: String,
    tx_hex: String,
) -> Result<Txid, SignAndSendError> {
    wallet::sign_and_send_swap_transaction(wallet_name, &LOADED_WALLET, tx_hex).await
}

/// Decomposes a transaction into:
///
/// - Sell amount, sell balance before and sell balance after.
/// - Buy amount, buy balance before and buy balance after.
///
/// To do so we unblind confidential `TxOut`s whenever necessary.
pub async fn extract_trade(wallet_name: String, transaction: String) -> Result<Trade> {
    let trade = wallet::extract_trade(wallet_name, &LOADED_WALLET, transaction).await?;

    Ok(trade)
}

/// Decomposes a loan into:
///
/// - Collateral amount, collateral asset balance before and collateral asset balance after.
/// - Principal amount, principal asset balance before and principal asset balance after.
/// - Principal repayment amount.
/// - Loan term.
///
/// To do so we unblind confidential `TxOut`s whenever necessary.
///
/// This also updates the state of the current loan protocol
/// "handshake" so that we can later on sign the loan transaction and
/// give it back to the lender.
pub async fn extract_loan(wallet_name: String, loan_response: String) -> Result<LoanDetails> {
    let details = wallet::extract_loan(wallet_name, &LOADED_WALLET, loan_response).await?;

    Ok(details)
}

pub async fn repay_loan(wallet_name: String, loan_txid: String) -> Result<Txid, RepayLoanError> {
    wallet::repay_loan(wallet_name, &LOADED_WALLET, loan_txid).await
}

#[cfg(test)]
mod constants_tests {
    use elements::{AddressParams, AssetId};
    use std::str::FromStr;

    #[test]
    fn assert_native_asset_ticker_constant() {
        match option_env!("NATIVE_ASSET_TICKER") {
            Some(native_asset_ticker) => {
                assert_eq!(crate::constants::NATIVE_ASSET_TICKER, native_asset_ticker)
            }
            None => assert_eq!(crate::constants::NATIVE_ASSET_TICKER, "L-BTC"),
        }
    }

    #[test]
    fn assert_native_asset_id_constant() {
        match option_env!("NATIVE_ASSET_ID") {
            Some(native_asset_id) => assert_eq!(
                *crate::constants::NATIVE_ASSET_ID,
                AssetId::from_str(native_asset_id).unwrap()
            ),
            None => assert_eq!(
                *crate::constants::NATIVE_ASSET_ID,
                AssetId::from_str(
                    "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d"
                )
                .unwrap()
            ),
        }
    }

    #[test]
    fn assert_usdt_asset_id_constant() {
        match option_env!("USDT_ASSET_ID") {
            Some(usdt_asset_id) => assert_eq!(
                *crate::constants::USDT_ASSET_ID,
                AssetId::from_str(usdt_asset_id).unwrap()
            ),
            None => assert_eq!(
                *crate::constants::USDT_ASSET_ID,
                AssetId::from_str(
                    "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2"
                )
                .unwrap()
            ),
        }
    }

    #[test]
    fn assert_esplora_api_url_constant() {
        match option_env!("ESPLORA_API_URL") {
            Some(esplora_api_url) => assert_eq!(crate::constants::ESPLORA_API_URL, esplora_api_url),
            None => assert_eq!(
                crate::constants::ESPLORA_API_URL,
                "https://blockstream.info/liquid/api"
            ),
        }
    }

    #[test]
    fn assert_address_params_constant() {
        match option_env!("CHAIN") {
            None | Some("LIQUID") => {
                assert_eq!(crate::constants::ADDRESS_PARAMS, &AddressParams::LIQUID)
            }
            Some("ELEMENTS") => {
                assert_eq!(crate::constants::ADDRESS_PARAMS, &AddressParams::ELEMENTS)
            }
            Some(chain) => panic!("unsupported chain {}", chain),
        }
    }

    #[test]
    fn assert_default_fee_constant() {
        let error_margin = f32::EPSILON;

        match option_env!("DEFAULT_SAT_PER_VBYTE") {
            Some(rate) => assert!(
                crate::constants::DEFAULT_SAT_PER_VBYTE - f32::from_str(rate).unwrap()
                    < error_margin
            ),
            None => assert!(crate::constants::DEFAULT_SAT_PER_VBYTE - 1.0f32 < error_margin),
        }
    }
}
