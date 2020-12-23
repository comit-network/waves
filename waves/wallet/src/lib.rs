use crate::{utils::set_panic_hook, wallet::Wallet};
use anyhow::{Context, Result};
use conquer_once::Lazy;
use elements_fun::{
    bitcoin::{Amount, Denomination},
    secp256k1::{All, Secp256k1},
};
use futures::lock::Mutex;
use js_sys::Array;
use wasm_bindgen::prelude::*;

#[macro_use]
mod macros;

mod cache_storage;
mod esplora;
mod storage;
mod typed_js_future;
mod utils;
mod wallet;

mod constants {
    include!(concat!(env!("OUT_DIR"), "/", "constants.rs"));
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static SECP: Lazy<Secp256k1<All>> = Lazy::new(Secp256k1::new);

static LOADED_WALLET: Lazy<Mutex<Option<Wallet>>> = Lazy::new(Mutex::default);

#[wasm_bindgen(start)]
pub fn setup_lib() {
    set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());
    log::debug!("wallet initialized");
}

/// Create a new wallet with the given name and password.
///
/// Fails if a wallet with this name already exists.
/// The created wallet will be automatically loaded.
#[wasm_bindgen]
pub async fn create_new_wallet(name: String, password: String) -> Result<(), JsValue> {
    wallet::create_new(name, password, &LOADED_WALLET).await
}

/// Load an existing wallet.
///
/// Fails if:
///
/// - the wallet does not exist
/// - the password is wrong
#[wasm_bindgen]
pub async fn load_existing_wallet(name: String, password: String) -> Result<(), JsValue> {
    wallet::load_existing(name, password, &LOADED_WALLET).await
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
    let status = wallet::get_status(name, &LOADED_WALLET).await?;

    Ok(JsValue::from_serde(&status).unwrap_throw())
}

/// Get an address for the wallet with the given name.
///
/// Fails if the wallet is currently not loaded.
#[wasm_bindgen]
pub async fn get_address(name: String) -> Result<String, JsValue> {
    let address = wallet::get_address(name, &LOADED_WALLET).await?;

    Ok(address.to_string())
}

/// Get the balances of the currently loaded wallet.
///
/// Returns an array of [`BalanceEntry`]s.
///
/// Fails if the wallet is currently not loaded or we cannot reach the block explorer for some reason.
#[wasm_bindgen]
pub async fn get_balances(name: String) -> Result<Array, JsValue> {
    let balances = wallet::get_balances(&name, &LOADED_WALLET).await?;

    Ok(balances
        .into_iter()
        .map(|e| JsValue::from_serde(&e).unwrap_throw())
        .collect::<Array>())
}

/// Withdraw all funds to the given address.
///
/// Returns the transaction ID of the transaction that was broadcasted.
#[wasm_bindgen]
pub async fn withdraw_everything_to(name: String, address: String) -> Result<String, JsValue> {
    let txid = wallet::withdraw_everything_to(
        name,
        &LOADED_WALLET,
        map_err_from_anyhow!(address
            .parse()
            .context("failed to parse address from string"))?,
    )
    .await?;

    Ok(txid.to_string())
}

/// Constructs a new [`CreateSwapPayload`] with the given Bitcoin amount.
///
/// This will select UTXOs from the wallet and compute an appropriate fee to spend them into a single output + a change output if necessary.
/// The other party (Bob) has to separately account for the cost of their transaction elements.
#[wasm_bindgen]
pub async fn make_create_swap_payload(
    wallet_name: String,
    btc: String,
) -> Result<JsValue, JsValue> {
    let payload = wallet::make_create_swap_payload(
        wallet_name,
        &LOADED_WALLET,
        map_err_from_anyhow!(Amount::from_str_in(&btc, Denomination::Bitcoin)
            .context("failed to parse amount from string"))?,
    )
    .await?;

    Ok(JsValue::from_serde(&payload).unwrap_throw())
}

#[cfg(test)]
mod constants_tests {
    use elements_fun::{AddressParams, AssetId};
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
                crate::constants::NATIVE_ASSET_ID,
                AssetId::from_str(native_asset_id).unwrap()
            ),
            None => assert_eq!(
                crate::constants::NATIVE_ASSET_ID,
                AssetId::from_str(
                    "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d"
                )
                .unwrap()
            ),
        }
    }

    #[test]
    fn assert_esplora_url_constant() {
        match option_env!("ESPLORA_URL") {
            Some(esplora_url) => assert_eq!(crate::constants::ELEMENTS_ESPLORA_URL, esplora_url),
            None => assert_eq!(
                crate::constants::ELEMENTS_ESPLORA_URL,
                "https://blockstream.info/liquid/api"
            ),
        }
    }

    #[test]
    fn assert_address_params_constant() {
        match option_env!("ELEMENTS_CHAIN") {
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
