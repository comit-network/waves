#[macro_use]
mod macros;

mod cache_storage;
mod esplora;
mod storage;
mod typed_js_future;
mod utils;
pub mod wallet;

use crate::{esplora::Utxo, utils::set_panic_hook, wallet::Wallet};
use anyhow::Result;
use conquer_once::Lazy;
use elements_fun::{
    secp256k1::{All, Secp256k1},
    AssetId,
};
use futures::{lock::Mutex, stream::FuturesUnordered, TryStreamExt};
use itertools::Itertools;
use js_sys::Array;
use wasm_bindgen::prelude::*;

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
    log::debug!("Wasm lib initialized");
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
pub async fn get_address(wallet_name: String) -> Result<String, JsValue> {
    let address = wallet::get_address(wallet_name, &LOADED_WALLET).await?;

    Ok(address.to_string())
}

pub async fn get_balances() -> Result<Array, JsValue> {
    let utxos = esplora::fetch_utxos(
        &"ex1qa2a790x3vl02uma5ndhcusf2r9dn0hd82f6jhf"
            .parse()
            .unwrap(),
    )
    .await
    .unwrap_throw();

    let utxos = utxos
        .into_iter()
        .map(|Utxo { txid, vout, .. }| async move {
            let tx = esplora::fetch_transaction(txid).await;

            tx.map(|mut tx| tx.output.remove(vout as usize))
        })
        .collect::<FuturesUnordered<_>>()
        .try_collect::<Vec<_>>()
        .await
        .unwrap_throw();

    let grouped_utxos = utxos
        .into_iter()
        .filter_map(|utxo| utxo.into_explicit()) // TODO: Unblind instead of just using explicit txouts
        .group_by(|explicit| explicit.asset);

    let balances = (&grouped_utxos)
        .into_iter()
        .map(|(asset, utxos)| {
            let balance_entry = BalanceEntry {
                value: utxos.map(|utxo| utxo.value.0).sum(),
                asset: asset.0,
            };

            JsValue::from_serde(&balance_entry).expect_throw("serialization always succeeds")
        })
        .collect::<Array>();

    Ok(balances)
}

#[derive(Debug, serde::Serialize)]
pub struct BalanceEntry {
    value: u64,
    asset: AssetId,
}
