use conquer_once::Lazy;
use futures::lock::Mutex;
use wasm_bindgen::prelude::*;

#[macro_use]
mod macros;

mod assets;
mod cache_storage;
mod esplora;
mod logger;
mod storage;
mod wallet;

use crate::storage::Storage;
pub use crate::wallet::*;

mod constants {
    include!(concat!(env!("OUT_DIR"), "/", "constants.rs"));
}

static LOADED_WALLET: Lazy<Mutex<Option<Wallet>>> = Lazy::new(Mutex::default);

#[wasm_bindgen(start)]
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
#[wasm_bindgen]
pub async fn create_new_wallet(name: String, password: String) -> Result<JsValue, JsValue> {
    map_err_from_anyhow!(wallet::create_new(name, password, &LOADED_WALLET).await)?;

    Ok(JsValue::null())
}

/// Load an existing wallet.
///
/// Fails if:
///
/// - the wallet does not exist
/// - the password is wrong
#[wasm_bindgen]
pub async fn load_existing_wallet(name: String, password: String) -> Result<JsValue, JsValue> {
    map_err_from_anyhow!(wallet::load_existing(name, password, &LOADED_WALLET).await)?;

    Ok(JsValue::null())
}

/// Unload the currently loaded wallet.
///
/// Does nothing if currently no wallet is loaded.
#[wasm_bindgen]
pub async fn unload_current_wallet() {
    wallet::unload_current(&LOADED_WALLET).await
}

/// Retrieve the status of the wallet with the given name.
#[wasm_bindgen]
pub async fn wallet_status(name: String) -> Result<JsValue, JsValue> {
    let status = map_err_from_anyhow!(wallet::get_status(name, &LOADED_WALLET).await)?;
    let status = map_err_from_anyhow!(JsValue::from_serde(&status))?;

    Ok(status)
}

/// Get an address for the wallet with the given name.
///
/// Fails if the wallet is currently not loaded.
#[wasm_bindgen]
pub async fn get_address(name: String) -> Result<JsValue, JsValue> {
    let address = map_err_from_anyhow!(wallet::get_address(name, &LOADED_WALLET).await)?;
    let address = map_err_from_anyhow!(JsValue::from_serde(&address))?;

    Ok(address)
}

/// Get the balances of the currently loaded wallet.
///
/// Returns an array of [`BalanceEntry`]s.
///
/// Fails if the wallet is currently not loaded or we cannot reach the block explorer for some reason.
#[wasm_bindgen]
pub async fn get_balances(name: String) -> Result<JsValue, JsValue> {
    let balance_entries = map_err_from_anyhow!(wallet::get_balances(&name, &LOADED_WALLET).await)?;
    let balance_entries = map_err_from_anyhow!(JsValue::from_serde(&balance_entries))?;

    Ok(balance_entries)
}

/// Withdraw all funds to the given address.
///
/// Returns the transaction ID of the transaction that was broadcasted.
#[wasm_bindgen]
pub async fn withdraw_everything_to(name: String, address: String) -> Result<JsValue, JsValue> {
    let txid =
        map_err_from_anyhow!(wallet::withdraw_everything_to(name, &LOADED_WALLET, address).await)?;
    let txid = map_err_from_anyhow!(JsValue::from_serde(&txid))?;

    Ok(txid)
}

/// Constructs a new [`CreateSwapPayload`] with the given USDt amount.
///
/// This will select UTXOs from the wallet to cover the given amount.
#[wasm_bindgen]
pub async fn make_buy_create_swap_payload(
    wallet_name: String,
    usdt: String,
) -> Result<JsValue, JsValue> {
    let payload = map_err_from_anyhow!(
        wallet::make_buy_create_swap_payload(wallet_name, &LOADED_WALLET, usdt).await
    )?;
    let payload = map_err_from_anyhow!(JsValue::from_serde(&payload))?;

    Ok(payload)
}

/// Constructs a new [`CreateSwapPayload`] with the given Bitcoin amount.
///
/// This will select UTXOs from the wallet to cover the given amount.
#[wasm_bindgen]
pub async fn make_sell_create_swap_payload(
    wallet_name: String,
    btc: String,
) -> Result<JsValue, JsValue> {
    let payload = map_err_from_anyhow!(
        wallet::make_sell_create_swap_payload(wallet_name, &LOADED_WALLET, btc).await
    )?;
    let payload = map_err_from_anyhow!(JsValue::from_serde(&payload))?;

    Ok(payload)
}

/// Constructs a new [`CreateSwapPayload`] with the given Bitcoin amount.
///
/// This will select UTXOs from the wallet to cover the given amount.
///
/// Additionally, sets the state of the loan protocol so that we can
/// continue after the lender sends back a response to our loan
/// request.
#[wasm_bindgen]
pub async fn make_loan_request(
    wallet_name: String,
    collateral: String,
) -> Result<JsValue, JsValue> {
    let loan_request = map_err_from_anyhow!(
        wallet::make_loan_request(wallet_name, &LOADED_WALLET, collateral).await
    )?;
    let loan_request = map_err_from_anyhow!(JsValue::from_serde(&loan_request))?;

    Ok(loan_request)
}

/// Sign a loan transaction in the wallet's state, if the state of the
/// current loan protocol allows it.
///
/// Returns the signed transaction.
#[wasm_bindgen]
pub async fn sign_loan(wallet_name: String) -> Result<JsValue, JsValue> {
    let loan_tx = map_err_from_anyhow!(wallet::sign_loan(wallet_name, &LOADED_WALLET).await)?;
    let loan_tx = map_err_from_anyhow!(JsValue::from_serde(&loan_tx))?;

    Ok(loan_tx)
}

/// Sign the given swap transaction and broadcast it to the network.
///
/// Returns the transaction ID.
#[wasm_bindgen]
pub async fn sign_and_send_swap_transaction(
    wallet_name: String,
    tx_hex: String,
) -> Result<JsValue, JsValue> {
    let txid = map_err_from_anyhow!(
        wallet::sign_and_send_swap_transaction(wallet_name, &LOADED_WALLET, tx_hex).await
    )?;
    let txid = map_err_from_anyhow!(JsValue::from_serde(&txid))?;

    Ok(txid)
}

/// Decomposes a transaction into:
///
/// - Sell amount, sell balance before and sell balance after.
/// - Buy amount, buy balance before and buy balance after.
///
/// To do so we unblind confidential `TxOut`s whenever necessary.
#[wasm_bindgen]
pub async fn extract_trade(wallet_name: String, transaction: String) -> Result<JsValue, JsValue> {
    let trade = map_err_from_anyhow!(
        wallet::extract_trade(wallet_name, &LOADED_WALLET, transaction).await
    )?;
    let trade = map_err_from_anyhow!(JsValue::from_serde(&trade))?;

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
#[wasm_bindgen]
pub async fn extract_loan(wallet_name: String, loan_response: JsValue) -> Result<JsValue, JsValue> {
    let loan_response = map_err_from_anyhow!(loan_response.into_serde())?;
    let details = map_err_from_anyhow!(
        wallet::extract_loan(wallet_name, &LOADED_WALLET, loan_response).await
    )?;
    let details = map_err_from_anyhow!(JsValue::from_serde(&details))?;

    Ok(details)
}

/// Returns all the active loans stored in the browser's local storage.
#[wasm_bindgen]
pub async fn get_open_loans() -> Result<JsValue, JsValue> {
    let storage = map_err_from_anyhow!(Storage::local_storage())?;
    let loans = map_err_from_anyhow!(storage.get_open_loans().await)?;
    let loans = map_err_from_anyhow!(JsValue::from_serde(&loans))?;

    Ok(loans)
}

#[wasm_bindgen]
pub async fn repay_loan(wallet_name: String, loan_txid: String) -> Result<JsValue, JsValue> {
    let txid =
        map_err_from_anyhow!(wallet::repay_loan(wallet_name, &LOADED_WALLET, loan_txid).await)?;
    let txid = map_err_from_anyhow!(JsValue::from_serde(&txid))?;

    Ok(txid)
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
