use std::str::FromStr;

use conquer_once::Lazy;
use elements::{
    bitcoin::util::amount::{Amount, Denomination},
    Address, AddressParams, Txid,
};
use futures::lock::Mutex;
use js_sys::Promise;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::window;

#[macro_use]
mod macros;

mod assets;
mod cache_storage;
mod esplora;
mod logger;
mod storage;
mod wallet;

use crate::{storage::Storage, wallet::*};
use reqwest::Url;

// TODO: make this configurable through extension option UI
const DEFAULT_SAT_PER_VBYTE: u64 = 1;

static LOADED_WALLET: Lazy<Mutex<Option<Wallet>>> = Lazy::new(Mutex::default);

// TODO: I was unable to use `futures::lock::Mutex` for these, but
// someone else should be able to do it
static CHAIN: Lazy<std::sync::Mutex<Chain>> = Lazy::new(|| {
    std::sync::Mutex::new(
        Storage::local_storage()
            .expect_throw("local storage to be available")
            .get_item::<Chain>("CHAIN")
            .expect_throw("failed to get 'CHAIN'")
            .expect_throw("empty 'CHAIN'"),
    )
});
static ESPLORA_API_URL: Lazy<std::sync::Mutex<Url>> = Lazy::new(|| {
    std::sync::Mutex::new(
        Storage::local_storage()
            .expect_throw("local storage to be available")
            .get_item::<Url>("ESPLORA_API_URL")
            .expect_throw("failed to get 'ESPLORA_API_URL'")
            .expect_throw("empty 'ESPLORA_API_URL'"),
    )
});
static BTC_ASSET_ID: Lazy<std::sync::Mutex<elements::AssetId>> = Lazy::new(|| {
    std::sync::Mutex::new(
        Storage::local_storage()
            .expect_throw("local storage to be available")
            .get_item::<elements::AssetId>("LBTC_ASSET_ID")
            .expect_throw("failed to get 'LBTC_ASSET_ID'")
            .expect_throw("empty 'LBTC_ASSET_ID'"),
    )
});
static USDT_ASSET_ID: Lazy<std::sync::Mutex<elements::AssetId>> = Lazy::new(|| {
    std::sync::Mutex::new(
        Storage::local_storage()
            .expect_throw("local storage to be available")
            .get_item::<elements::AssetId>("LUSDT_ASSET_ID")
            .expect_throw("failed to get 'LUSDT_ASSET_ID'")
            .expect_throw("empty 'LUSDT_ASSET_ID'"),
    )
});

#[wasm_bindgen(start)]
pub fn setup() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    logger::try_init();
    log::info!("wallet initialized");

    let handler = Closure::wrap(
        Box::new(handle_storage_update) as Box<dyn Fn(web_sys::StorageEvent) -> Promise>
    );
    let window = window().unwrap();
    window
        .add_event_listener_with_callback("storage", handler.as_ref().unchecked_ref())
        .unwrap();
    handler.forget();
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
    let address = map_err_from_anyhow!(address.parse::<Address>())?;
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
    let usdt = map_err_from_anyhow!(Amount::from_str_in(&usdt, Denomination::Bitcoin))?;
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
    let btc = map_err_from_anyhow!(Amount::from_str_in(&btc, Denomination::Bitcoin))?;
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
    let collateral = map_err_from_anyhow!(Amount::from_str_in(&collateral, Denomination::Bitcoin))?;
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
    let loan_tx = map_err_from_anyhow!(JsValue::from_serde(&Transaction::from(loan_tx)))?;

    Ok(loan_tx)
}

/// Sign the given swap transaction and broadcast it to the network.
///
/// Returns the transaction ID.
#[wasm_bindgen]
pub async fn sign_and_send_swap_transaction(
    wallet_name: String,
    transaction: JsValue,
) -> Result<JsValue, JsValue> {
    let transaction: Transaction = map_err_from_anyhow!(transaction.into_serde())?;
    let txid = map_err_from_anyhow!(
        wallet::sign_and_send_swap_transaction(wallet_name, &LOADED_WALLET, transaction.into())
            .await
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
pub async fn extract_trade(wallet_name: String, transaction: JsValue) -> Result<JsValue, JsValue> {
    let transaction: Transaction = map_err_from_anyhow!(transaction.into_serde())?;
    let trade = map_err_from_anyhow!(
        wallet::extract_trade(wallet_name, &LOADED_WALLET, transaction.into()).await
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
    let loan_txid = map_err_from_anyhow!(Txid::from_str(&loan_txid))?;
    let txid =
        map_err_from_anyhow!(wallet::repay_loan(wallet_name, &LOADED_WALLET, loan_txid).await)?;
    let txid = map_err_from_anyhow!(JsValue::from_serde(&txid))?;

    Ok(txid)
}

#[wasm_bindgen]
pub async fn get_past_transactions(wallet_name: String) -> Result<JsValue, JsValue> {
    let history =
        map_err_from_anyhow!(wallet::get_transaction_history(wallet_name, &LOADED_WALLET).await)?;
    let history = map_err_from_anyhow!(JsValue::from_serde(&history))?;

    Ok(history)
}

fn handle_storage_update(event: web_sys::StorageEvent) -> Promise {
    match (event.key().as_deref(), event.new_value().as_deref()) {
        (Some("CHAIN"), Some(new_value)) => {
            let mut guard = CHAIN.lock().expect_throw("could not acquire lock");
            *guard = Chain::from_str(&new_value)
                .expect_throw(&format!("could not parse item: {}", new_value));
        }
        (Some("ESPLORA_API_URL"), Some(new_value)) => {
            let esplora_api_url = match Url::parse(new_value) {
                Ok(esplora_api_url) => esplora_api_url,
                Err(e) => {
                    let error_msg = format!("Could not get item 'ESPLORA_API_URL' {}", e);
                    return Promise::reject(&JsValue::from_str(error_msg.as_str()));
                }
            };

            let mut guard = ESPLORA_API_URL
                .lock()
                .expect_throw("could not acquire lock.");
            *guard = esplora_api_url;
        }
        (Some("LBTC_ASSET_ID"), Some(new_value)) => {
            let mut guard = BTC_ASSET_ID.lock().expect_throw("could not acquire lock");
            *guard = elements::AssetId::from_str(new_value)
                .expect_throw(&format!("could not parse item: {}", new_value));
        }
        (Some("LUSDT_ASSET_ID"), Some(new_value)) => {
            let mut guard = USDT_ASSET_ID.lock().expect_throw("could not acquire lock");
            *guard = elements::AssetId::from_str(new_value)
                .expect_throw(&format!("could not parse item: {}", new_value));
        }
        _ => {
            log::trace!("Storage event not handled! {:?}", event.key());
        }
    };
    Promise::resolve(&JsValue::null())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Chain {
    Elements,
    Liquid,
}

impl From<Chain> for &AddressParams {
    fn from(from: Chain) -> Self {
        match from {
            Chain::Elements => &AddressParams::ELEMENTS,
            Chain::Liquid => &AddressParams::LIQUID,
        }
    }
}

impl FromStr for Chain {
    type Err = WrongChain;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowercase = s.to_ascii_lowercase();
        match lowercase.as_str() {
            "elements" => Ok(Chain::Elements),
            "liquid" => Ok(Chain::Liquid),
            _ => Err(WrongChain(lowercase)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Unsupported chain: {0}")]
struct WrongChain(String);

#[derive(serde::Serialize, serde::Deserialize)]
struct Transaction {
    #[serde(with = "baru::loan::transaction_as_string")]
    inner: elements::Transaction,
}

impl From<elements::Transaction> for Transaction {
    fn from(from: elements::Transaction) -> Self {
        Self { inner: from }
    }
}

impl From<Transaction> for elements::Transaction {
    fn from(from: Transaction) -> Self {
        from.inner
    }
}
