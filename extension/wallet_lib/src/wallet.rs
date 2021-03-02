use crate::{
    assets::lookup,
    constants::{ADDRESS_PARAMS, DEFAULT_SAT_PER_VBYTE, NATIVE_ASSET_ID, NATIVE_ASSET_TICKER},
    esplora,
    esplora::Utxo,
};
use aes_gcm_siv::{
    aead::{Aead, NewAead},
    Aes256GcmSiv,
};
use anyhow::{bail, Context, Result};
use elements::{
    bitcoin,
    bitcoin::secp256k1::{SecretKey, SECP256K1},
    confidential,
    secp256k1::{rand, PublicKey},
    Address, AssetId, OutPoint, TxOut,
};
use futures::{
    lock::{MappedMutexGuard, Mutex, MutexGuard},
    stream::FuturesUnordered,
    StreamExt, TryStreamExt,
};
use hkdf::Hkdf;
use itertools::Itertools;
use rand::{thread_rng, Rng};
use rust_decimal::Decimal;
use sha2::{digest::generic_array::GenericArray, Sha256};
use std::{fmt, str};

pub use create_new::create_new;
pub use extract_trade::{extract_trade, Trade};
pub use get_address::get_address;
pub use get_balances::get_balances;
pub use get_status::{get_status, WalletStatus};
pub use load_existing::load_existing;
pub use make_create_swap_payload::{make_buy_create_swap_payload, make_sell_create_swap_payload};
pub use sign_and_send_swap_transaction::sign_and_send_swap_transaction;
use std::convert::Infallible;
pub use unload_current::unload_current;
pub use withdraw_everything_to::withdraw_everything_to;

mod coin_selection;
mod create_new;
mod extract_trade;
mod get_address;
mod get_balances;
mod get_status;
mod load_existing;
mod make_create_swap_payload;
mod sign_and_send_swap_transaction;
mod unload_current;
mod withdraw_everything_to;

async fn get_txouts<T, FM: Fn(Utxo, TxOut) -> Result<Option<T>> + Copy>(
    wallet: &Wallet,
    filter_map: FM,
) -> Result<Vec<T>> {
    let address = wallet.get_address();

    let utxos = esplora::fetch_utxos(&address).await?;

    let txouts = utxos
        .into_iter()
        .map(move |utxo| async move {
            let mut tx = esplora::fetch_transaction(utxo.txid).await?;
            let txout = tx.output.remove(utxo.vout as usize);

            filter_map(utxo, txout)
        })
        .collect::<FuturesUnordered<_>>()
        .filter_map(|r| std::future::ready(r.transpose()))
        .try_collect::<Vec<_>>()
        .await?;

    Ok(txouts)
}

async fn current<'n, 'w>(
    name: &'n str,
    current_wallet: &'w Mutex<Option<Wallet>>,
) -> Result<MappedMutexGuard<'w, Option<Wallet>, Wallet>> {
    let mut guard = current_wallet.lock().await;

    match &mut *guard {
        Some(wallet) if wallet.name == name => {}
        _ => bail!("wallet with name '{}' is currently not loaded", name),
    };

    Ok(MutexGuard::map(guard, |w| w.as_mut().unwrap()))
}

#[derive(Debug)]
pub struct Wallet {
    name: String,
    encryption_key: [u8; 32],
    secret_key: SecretKey,
    sk_salt: [u8; 32],
}

const SECRET_KEY_ENCRYPTION_NONCE: &[u8; 12] = b"SECRET_KEY!!";

impl Wallet {
    pub fn initialize_new(name: String, password: String, secret_key: SecretKey) -> Result<Self> {
        let sk_salt = thread_rng().gen::<[u8; 32]>();

        let encryption_key = Self::derive_encryption_key(&password, &sk_salt)?;

        Ok(Self {
            name,
            encryption_key,
            secret_key,
            sk_salt,
        })
    }

    pub fn initialize_existing(
        name: String,
        password: String,
        sk_ciphertext: String,
    ) -> Result<Self> {
        let mut parts = sk_ciphertext.split('$');

        let salt = parts.next().context("no salt in cipher text")?;
        let sk = parts.next().context("no secret key in cipher text")?;

        let mut sk_salt = [0u8; 32];
        hex::decode_to_slice(salt, &mut sk_salt).context("failed to decode salt as hex")?;

        let encryption_key = Self::derive_encryption_key(&password, &sk_salt)?;

        let cipher = Aes256GcmSiv::new(GenericArray::from_slice(&encryption_key));
        let nonce = GenericArray::from_slice(SECRET_KEY_ENCRYPTION_NONCE);
        let sk = cipher
            .decrypt(
                nonce,
                hex::decode(sk)
                    .context("failed to decode sk as hex")?
                    .as_slice(),
            )
            .context("failed to decrypt secret key")?;

        Ok(Self {
            name,
            encryption_key,
            secret_key: SecretKey::from_slice(&sk).context("invalid secret key")?,
            sk_salt,
        })
    }

    pub fn get_public_key(&self) -> PublicKey {
        PublicKey::from_secret_key(SECP256K1, &self.secret_key)
    }

    pub fn get_address(&self) -> Address {
        let public_key = self.get_public_key();
        let blinding_key = PublicKey::from_secret_key(SECP256K1, &self.blinding_key());

        Address::p2wpkh(
            &bitcoin::PublicKey {
                compressed: true,
                key: public_key,
            },
            Some(blinding_key),
            &ADDRESS_PARAMS,
        )
    }

    /// Encrypts the secret key with the encryption key.
    ///
    /// # Choice of nonce
    ///
    /// We store the secret key on disk and as such have to use a constant nonce, otherwise we would not be able to decrypt it again.
    /// The encryption only happens once and as such, there is conceptually only one message and we are not "reusing" the nonce which would be insecure.
    fn encrypted_secret_key(&self) -> Result<Vec<u8>> {
        let cipher = Aes256GcmSiv::new(&GenericArray::from_slice(&self.encryption_key));
        let enc_sk = cipher
            .encrypt(
                GenericArray::from_slice(SECRET_KEY_ENCRYPTION_NONCE),
                &self.secret_key[..],
            )
            .context("failed to encrypt secret key")?;

        Ok(enc_sk)
    }

    /// Derive the blinding key.
    ///
    /// # Choice of salt
    ///
    /// We choose to not add a salt because the ikm is already a randomly-generated, secret value with decent entropy.
    ///
    /// # Choice of ikm
    ///
    /// We derive the blinding key from the secret key to avoid having to store two secret values on disk.
    ///
    /// # Choice of info
    ///
    /// We choose to tag the derived key with `b"BLINDING_KEY"` in case we ever want to derive something else from the secret key.
    fn blinding_key(&self) -> SecretKey {
        let h = Hkdf::<sha2::Sha256>::new(None, self.secret_key.as_ref());

        let mut bk = [0u8; 32];
        h.expand(b"BLINDING_KEY", &mut bk)
            .expect("output length aligns with sha256");

        SecretKey::from_slice(bk.as_ref()).expect("always a valid secret key")
    }

    /// Derive the encryption key from the wallet's password and a salt.
    ///
    /// # Choice of salt
    ///
    /// The salt of HKDF can be public or secret and while it can operate without a salt, it is better to pass a salt value [0].
    ///
    /// # Choice of ikm
    ///
    /// The user's password is our input key material. The stronger the password, the better the resulting encryption key.
    ///
    /// # Choice of info
    ///
    /// HKDF can operate without `info`, however, it is useful to "tag" the derived key with its usage.
    /// In our case, we use the encryption key to encrypt the secret key and as such, tag it with `b"ENCRYPTION_KEY"`.
    ///
    /// [0]: https://tools.ietf.org/html/rfc5869#section-3.1
    fn derive_encryption_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
        let h = Hkdf::<Sha256>::new(Some(salt), password.as_bytes());
        let mut enc_key = [0u8; 32];
        h.expand(b"ENCRYPTION_KEY", &mut enc_key)
            .context("failed to derive encryption key")?;

        Ok(enc_key)
    }
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
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateSwapPayload {
    pub alice_inputs: Vec<SwapUtxo>,
    pub address: Address,
    #[serde(with = "bdk::bitcoin::util::amount::serde::as_sat")]
    pub amount: bdk::bitcoin::Amount,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SwapUtxo {
    pub outpoint: OutPoint,
    pub blinding_key: SecretKey,
}

/// A single balance entry as returned by [`get_balances`].
#[derive(Debug, serde::Serialize)]
pub struct BalanceEntry {
    pub asset: AssetId,
    pub ticker: String,
    pub value: Decimal,
}

impl BalanceEntry {
    pub fn for_asset(asset: AssetId, ticker: String, value: u64, precision: u32) -> Self {
        let mut decimal = Decimal::from(value);
        decimal
            .set_scale(precision)
            .expect("precision must be < 28");

        Self {
            asset,
            ticker,
            value: decimal,
        }
    }
}

/// A pure function to compute the balances of the wallet given a set of [`TxOut`]s.
fn compute_balances(wallet: &Wallet, txouts: &[TxOut]) -> Vec<BalanceEntry> {
    let grouped_txouts = txouts
        .iter()
        .filter_map(|utxo| match utxo {
            TxOut {
                asset: confidential::Asset::Explicit(asset),
                value: confidential::Value::Explicit(value),
                ..
            } => Some((*asset, *value)),
            txout => {
                let confidential = match txout.to_confidential() {
                    Some(confidential) => confidential,
                    None => return None,
                };

                match confidential.unblind(SECP256K1, wallet.blinding_key()) {
                    Ok(unblinded_txout) => Some((unblinded_txout.asset, unblinded_txout.value)),
                    Err(e) => {
                        log::warn!("failed to unblind txout: {}", e);
                        None
                    }
                }
            }
        })
        .into_group_map();

    grouped_txouts
        .into_iter()
        .filter_map(|(asset, utxos)| {
            let total_sum = utxos.into_iter().sum();
            let (ticker, precision) = lookup(asset)?;

            Some(BalanceEntry::for_asset(
                asset,
                ticker.to_owned(),
                total_sum,
                precision as u32,
            ))
        })
        .collect()
}

/// These constants have been reverse engineered through the following transactions:
///
/// https://blockstream.info/liquid/tx/a17f4063b3a5fdf46a7012c82390a337e9a0f921933dccfb8a40241b828702f2
/// https://blockstream.info/liquid/tx/d12ff4e851816908810c7abc839dd5da2c54ad24b4b52800187bee47df96dd5c
/// https://blockstream.info/liquid/tx/47e60a3bc5beed45a2cf9fb7a8d8969bab4121df98b0034fb0d44f6ed2d60c7d
///
/// This gives us the following set of linear equations:
///
/// - 1 in, 1 out, 1 fee = 1332
/// - 1 in, 2 out, 1 fee = 2516
/// - 2 in, 2 out, 1 fee = 2623
///
/// Which we can solve using wolfram alpha: https://www.wolframalpha.com/input/?i=1x+%2B+1y+%2B+1z+%3D+1332%2C+1x+%2B+2y+%2B+1z+%3D+2516%2C+2x+%2B+2y+%2B+1z+%3D+2623
pub mod avg_vbytes {
    pub const INPUT: u64 = 107;
    pub const OUTPUT: u64 = 1184;
    pub const FEE: u64 = 41;
}

/// Estimate the virtual size of a transaction based on the number of inputs and outputs.
pub fn estimate_virtual_size(number_of_inputs: u64, number_of_outputs: u64) -> u64 {
    number_of_inputs * avg_vbytes::INPUT + number_of_outputs * avg_vbytes::OUTPUT + avg_vbytes::FEE
}

#[cfg(all(test, target_arch = "wasm32"))]
mod browser_tests {
    use wasm_bindgen_test::*;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    pub async fn given_no_wallet_when_getting_address_then_fails() {
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

        let error = load_existing("wallet-4".to_owned(), "foo".to_owned(), &current_wallet)
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

        let error = load_existing("wallet-6".to_owned(), "bar".to_owned(), &current_wallet)
            .await
            .unwrap_err();

        assert_eq!(error.to_string(), "bad password for wallet 'wallet-6'");
    }

    #[wasm_bindgen_test]
    pub async fn cannot_load_wallet_that_doesnt_exist() {
        let current_wallet = Mutex::default();

        let error = load_existing("foobar".to_owned(), "bar".to_owned(), &current_wallet)
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

            wallet.secret_key.clone()
        };

        unload_current(&current_wallet).await;

        load_existing("wallet-9".to_owned(), "foo".to_owned(), &current_wallet)
            .await
            .unwrap();
        let loaded_sk = {
            let guard = current_wallet.lock().await;
            let wallet = guard.as_ref().unwrap();

            wallet.secret_key.clone()
        };

        assert_eq!(initial_sk, loaded_sk);
    }
}
