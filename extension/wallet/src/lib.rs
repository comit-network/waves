use anyhow::{Context, Result};
use baru::loan::LoanResponse;
use bip32::{Language, Mnemonic};
use conquer_once::Lazy;
use elements::{bitcoin::util::amount::Amount, encode::serialize_hex, Address, AddressParams};
use futures::{lock::Mutex, TryFutureExt};
use js_sys::Promise;
use reqwest::Url;
use runtime::browser;
use rust_decimal::{prelude::ToPrimitive, Decimal, RoundingStrategy};
use serde::Deserialize;
use std::str::FromStr;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::future_to_promise;
use web_sys::window;

#[macro_use]
mod macros;

mod assets;
mod cache_storage;
mod esplora;
mod logger;
mod runtime;
mod storage;
mod wallet;

use crate::{storage::Storage, wallet::*};

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

#[wasm_bindgen]
pub fn initialize() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    logger::try_init();

    let handler = Closure::wrap(
        Box::new(handle_storage_update) as Box<dyn Fn(web_sys::StorageEvent) -> Promise>
    );
    let window = window().unwrap();
    window
        .add_event_listener_with_callback("storage", handler.as_ref().unchecked_ref())
        .unwrap();
    handler.forget();

    add_message_handler!(
        browser,
        async fn walletStatus() -> Result<WalletStatus> {
            let status = wallet::get_status("demo".to_owned(), &LOADED_WALLET).await?;

            Ok(status)
        }
    );
    add_message_handler!(
        browser,
        async fn getNewAddress() -> Result<elements::Address> {
            let address = wallet::get_address("demo".to_owned(), &LOADED_WALLET).await?;

            Ok(address)
        }
    );
    add_message_handler!(
        browser,
        async fn makeSellCreateSwapPayload(btc: String) -> Result<CreateSwapPayload> {
            let btc = parse_to_bitcoin_amount(btc)?;
            let payload =
                wallet::make_sell_create_swap_payload("demo".to_owned(), &LOADED_WALLET, btc)
                    .await?;

            Ok(payload)
        }
    );
    add_message_handler!(
        browser,
        async fn makeBuyCreateSwapPayload(usdt: String) -> Result<CreateSwapPayload> {
            let usdt = parse_to_bitcoin_amount(usdt)?;
            let payload =
                wallet::make_buy_create_swap_payload("demo".to_owned(), &LOADED_WALLET, usdt)
                    .await?;

            Ok(payload)
        }
    );
    add_message_handler!(
        browser,
        async fn makeLoanRequestPayload(
            collateral: String,
            fee_rate: String,
        ) -> Result<LoanRequest> {
            // TODO: Change the UI to handle SATs not BTC
            let collateral_in_btc = parse_to_bitcoin_amount(collateral)?;
            let fee_rate_in_sat = Amount::from_sat(u64::from_str(fee_rate.as_str())?);
            let payload = wallet::make_loan_request(
                "demo".to_owned(),
                &LOADED_WALLET,
                collateral_in_btc,
                fee_rate_in_sat,
            )
            .await?;

            Ok(payload)
        }
    );

    impl_window!(
        window,
        async fn extractTrade(hex: String) -> Result<Trade> {
            let transaction = deserialize_hex(&hex)?;
            let trade =
                wallet::extract_trade("demo".to_owned(), &LOADED_WALLET, transaction).await?;

            Ok(trade)
        }
    );
    impl_window!(
        window,
        async fn extractLoan(response: LoanResponse) -> Result<LoanDetails> {
            let loan = wallet::extract_loan("demo".to_owned(), &LOADED_WALLET, response).await?;

            Ok(loan)
        }
    );
    impl_window!(
        window,
        async fn signAndSendSwap(hex: String) -> Result<elements::TxId> {
            let transaction = deserialize_hex::<elements::Transaction>(&hex)?;
            let txid = wallet::sign_and_send_swap_transaction(
                "demo".to_owned(),
                &LOADED_WALLET,
                transaction,
            )
            .await?;

            Ok(txid)
        }
    );
    impl_window!(
        window,
        async fn unlockWallet(password: String) -> Result<()> {
            wallet::load_existing("demo".to_owned(), password, &LOADED_WALLET).await?;

            Ok(())
        }
    );
    impl_window!(
        window,
        async fn withdrawAll(address: Address) -> Result<elements::TxId> {
            let txid =
                wallet::withdraw_everything_to("demo".to_owned(), &LOADED_WALLET, address).await?;

            Ok(txid)
        }
    );
    impl_window!(
        window,
        async fn getWalletStatus() -> Result<WalletStatus> {
            let status = wallet::get_status("demo".to_owned(), &LOADED_WALLET).await?;

            Ok(status)
        }
    );
    impl_window!(
        window,
        async fn getBalances() -> Result<Vec<BalanceEntry>> {
            let balances = wallet::get_balances("demo".to_owned(), &LOADED_WALLET).await?;

            Ok(balances)
        }
    );
    impl_window!(
        window,
        async fn createNewWallet(seed_words: String, password: String) -> Result<()> {
            let mnemonic = Mnemonic::new(seed_words, Language::English)
                .map_err(|_| anyhow::anyhow!("Failed to parse seed words"))?;
            wallet::create_from_bip39("demo".to_owned(), mnemonic, password, &LOADED_WALLET)
                .await?;

            Ok(())
        }
    );
    impl_window!(
        window,
        async fn repayLoan(txid: String) -> Result<elements::Txid> {
            let loan_txid = txid.parse()?;
            let repay_txid =
                wallet::repay_loan("demo".to_owned(), &LOADED_WALLET, loan_txid).await?;

            Ok(repay_txid)
        }
    );
    impl_window!(
        window,
        async fn getAddress() -> Result<elements::Address> {
            let address = wallet::get_address("demo".to_owned(), &LOADED_WALLET).await?;

            Ok(address)
        }
    );
    impl_window!(
        window,
        async fn signLoan() -> Result<String> {
            let transaction = wallet::sign_loan("demo".to_owned(), &LOADED_WALLET).await?;
            let hex = serialize_hex(&transaction);

            Ok(hex)
        }
    );
    impl_window!(
        window,
        async fn getOpenLoans() -> Result<Vec<LoanDetails>> {
            let loans = Storage::local_storage()?.get_open_loans().await?;

            Ok(loans)
        }
    );
    impl_window!(
        window,
        async fn createLoanBackup(transaction: String) -> Result<Vec<LoanDetails>> {
            let transaction = deserialize_hex::<elements::Transaction>(&transaction)?;
            let backup =
                wallet::create_loan_backup("demo".to_owned(), &LOADED_WALLET, transaction.txid())
                    .await?;

            Ok(backup)
        }
    );
    impl_window!(
        window,
        async fn loadLoanBackup(backup: BackupDetails) -> Result<()> {
            // FIXME: The fact that this doesn't use the current wallet is a code smell that the storage schema is bad.
            wallet::load_loan_backup(backup)?;

            Ok(())
        }
    );
    impl_window!(
        window,
        async fn generateBip39SeedWords() -> Result<String> {
            let mnemonic = wallet::bip39_seed_words(Language::English);
            let words = mnemonic.phrase().to_owned();

            Ok(words)
        }
    );

    log::info!("WASM event listeners initialized");
}

fn handle_storage_update(event: web_sys::StorageEvent) -> Promise {
    match (event.key().as_deref(), event.new_value().as_deref()) {
        (Some("CHAIN"), Some(new_value)) => {
            let mut guard = CHAIN.lock().expect_throw("could not acquire lock");
            *guard = Chain::from_str(new_value)
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

/// Companion function to [`elements::encode::serialize_hex`] which unfortunately doesn't exist upstream.
fn deserialize_hex<T>(string: &str) -> Result<T>
where
    T: elements::encode::Decodable,
{
    let bytes = hex::decode(string)?;
    let t = elements::encode::deserialize::<T>(&bytes)?;

    Ok(t)
}

fn parse_to_bitcoin_amount(amount: String) -> Result<Amount> {
    let parsed = Decimal::from_str(amount.as_str())?;
    let rounded = parsed
        .round_dp_with_strategy(8, RoundingStrategy::MidpointAwayFromZero)
        .to_f64()
        .context("decimal cannot be represented as f64")?;
    let amount = Amount::from_btc(rounded)?;

    Ok(amount)
}
