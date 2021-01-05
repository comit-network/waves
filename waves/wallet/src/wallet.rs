use crate::{
    constants::{ADDRESS_PARAMS, DEFAULT_SAT_PER_VBYTE, NATIVE_ASSET_ID, NATIVE_ASSET_TICKER},
    esplora,
    esplora::Utxo,
};
use aes_gcm_siv::{
    aead::{Aead, NewAead},
    Aes256GcmSiv,
};
use anyhow::{Context, Result};
use elements_fun::{
    bitcoin,
    bitcoin::{
        secp256k1::{SecretKey, SECP256K1},
        Amount,
    },
    secp256k1::{rand, PublicKey},
    Address, OutPoint, TxOut,
};
use futures::{
    lock::{MappedMutexGuard, Mutex, MutexGuard},
    stream::FuturesUnordered,
    StreamExt, TryStreamExt,
};
use hkdf::Hkdf;
use rand::{thread_rng, Rng};
use sha2::{digest::generic_array::GenericArray, Sha256};
use std::{fmt, str};
use wasm_bindgen::{JsValue, UnwrapThrowExt};

pub use create_new::create_new;
pub use get_address::get_address;
pub use get_balances::get_balances;
pub use get_status::get_status;
pub use load_existing::load_existing;
pub use make_create_sell_swap_payload::make_create_sell_swap_payload;
pub use sign_and_send_swap_transaction::sign_and_send_swap_transaction;
pub use unload_current::unload_current;
pub use withdraw_everything_to::withdraw_everything_to;

mod coin_selection;
mod create_new;
mod get_address;
mod get_balances;
mod get_status;
mod load_existing;
mod make_create_sell_swap_payload;
mod sign_and_send_swap_transaction;
mod unload_current;
mod withdraw_everything_to;

async fn get_txouts<T, FM: Fn(Utxo, TxOut) -> Result<Option<T>> + Copy>(
    wallet: &Wallet,
    filter_map: FM,
) -> Result<Vec<T>, JsValue> {
    let address = wallet.get_address()?;

    let utxos = map_err_from_anyhow!(esplora::fetch_utxos(&address).await)?;

    let txouts = map_err_from_anyhow!(
        utxos
            .into_iter()
            .map(move |utxo| async move {
                let mut tx = esplora::fetch_transaction(utxo.txid).await?;
                let txout = tx.output.remove(utxo.vout as usize);

                filter_map(utxo, txout)
            })
            .collect::<FuturesUnordered<_>>()
            .filter_map(|r| std::future::ready(r.transpose()))
            .try_collect::<Vec<_>>()
            .await
    )?;

    Ok(txouts)
}

async fn current<'n, 'w>(
    name: &'n str,
    current_wallet: &'w Mutex<Option<Wallet>>,
) -> Result<MappedMutexGuard<'w, Option<Wallet>, Wallet>, JsValue> {
    let mut guard = current_wallet.lock().await;

    match &mut *guard {
        Some(wallet) if wallet.name == name => {}
        _ => {
            return Err(JsValue::from_str(&format!(
                "wallet with name '{}' is currently not loaded",
                name
            )))
        }
    };

    Ok(MutexGuard::map(guard, |w| w.as_mut().unwrap_throw()))
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
    pub fn initialize_new(
        name: String,
        password: String,
        secret_key: SecretKey,
    ) -> Result<Self, JsValue> {
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
    ) -> Result<Self, JsValue> {
        let mut parts = sk_ciphertext.split('$');

        let salt = map_err_from_anyhow!(parts.next().context("no salt in cipher text"))?;
        let sk = map_err_from_anyhow!(parts.next().context("no secret key in cipher text"))?;

        let mut sk_salt = [0u8; 32];
        map_err_from_anyhow!(
            hex::decode_to_slice(salt, &mut sk_salt).context("failed to decode salt as hex")
        )?;

        let encryption_key = Self::derive_encryption_key(&password, &sk_salt)?;

        let cipher = Aes256GcmSiv::new(GenericArray::from_slice(&encryption_key));
        let nonce = GenericArray::from_slice(SECRET_KEY_ENCRYPTION_NONCE);
        let sk = map_err_from_anyhow!(cipher
            .decrypt(
                nonce,
                map_err_from_anyhow!(hex::decode(sk).context("failed to decode sk as hex"))?
                    .as_slice()
            )
            .context("failed to decrypt secret key"))?;

        Ok(Self {
            name,
            encryption_key,
            secret_key: map_err_from_anyhow!(
                SecretKey::from_slice(&sk).context("invalid secret key")
            )?,
            sk_salt,
        })
    }

    pub fn get_public_key(&self) -> PublicKey {
        PublicKey::from_secret_key(SECP256K1, &self.secret_key)
    }

    pub fn get_address(&self) -> Result<Address, JsValue> {
        let public_key = self.get_public_key();
        let blinding_key = PublicKey::from_secret_key(SECP256K1, &self.blinding_key());

        let address = Address::p2wpkh(
            &bitcoin::PublicKey {
                compressed: true,
                key: public_key,
            },
            Some(blinding_key),
            &ADDRESS_PARAMS,
        );

        Ok(address)
    }

    /// Encrypts the secret key with the encryption key.
    ///
    /// # Choice of nonce
    ///
    /// We store the secret key on disk and as such have to use a constant nonce, otherwise we would not be able to decrypt it again.
    /// The encryption only happens once and as such, there is conceptually only one message and we are not "reusing" the nonce which would be insecure.
    fn encrypted_secret_key(&self) -> Result<Vec<u8>, JsValue> {
        let cipher = Aes256GcmSiv::new(&GenericArray::from_slice(&self.encryption_key));
        let enc_sk = map_err_from_anyhow!(cipher
            .encrypt(
                GenericArray::from_slice(SECRET_KEY_ENCRYPTION_NONCE),
                &self.secret_key[..]
            )
            .context("failed to encrypt secret key"))?;

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
    fn derive_encryption_key(password: &str, salt: &[u8]) -> Result<[u8; 32], JsValue> {
        let h = Hkdf::<Sha256>::new(Some(salt), password.as_bytes());
        let mut enc_key = [0u8; 32];
        map_err_from_anyhow!(h
            .expand(b"ENCRYPTION_KEY", &mut enc_key)
            .context("failed to derive encryption key"))?;

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
    type Err = String;

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
#[derive(Debug, serde::Serialize)]
pub struct CreateSwapPayload {
    pub alice_inputs: Vec<SwapUtxo>,
    pub address: Address,
    #[serde(with = "elements_fun::bitcoin::util::amount::serde::as_sat")]
    pub btc_amount: Amount,
}

#[derive(Debug, serde::Serialize)]
pub struct SwapUtxo {
    pub outpoint: OutPoint,
    pub blinding_key: SecretKey,
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::{
        esplora::{AssetDescription, AssetStatus},
        wallet::get_balances::BalanceEntry,
    };
    use elements_fun::AssetId;

    #[test]
    fn new_balance_entry_bitcoin_serializes_to_nominal_representation() {
        let liquid_native_ad = AssetDescription::default(AssetId::default());
        let entry = BalanceEntry::for_asset(100_000_000, liquid_native_ad, "L-BTC");

        let serialized = serde_json::to_string(&entry).unwrap();

        assert_eq!(
            r#"{"value":"1.00000000","asset":"0000000000000000000000000000000000000000000000000000000000000000","ticker":"L-BTC"}"#,
            serialized
        );
    }

    #[test]
    fn new_balance_entry_usdt_serializes_to_nominal_representation() {
        let usdt_ad = AssetDescription {
            asset_id: AssetId::default(),
            ticker: Some("L-USDT".to_string()),
            precision: Some(8),
            status: Some(AssetStatus { confirmed: true }),
        };
        let entry = BalanceEntry::for_asset(100_000_000, usdt_ad, "L-BTC");

        let serialized = serde_json::to_string(&entry).unwrap();

        assert_eq!(
            r#"{"value":"1.00000000","asset":"0000000000000000000000000000000000000000000000000000000000000000","ticker":"L-USDT"}"#,
            serialized
        );
    }

    #[test]
    fn new_balance_entry_custom_asset_serializes_to_nominal_representation() {
        let usdt_ad = AssetDescription {
            asset_id: AssetId::default(),
            ticker: Some("FOO".to_string()),
            precision: Some(3),
            status: Some(AssetStatus { confirmed: true }),
        };
        let entry = BalanceEntry::for_asset(100_000_000, usdt_ad, "L-BTC");

        let serialized = serde_json::to_string(&entry).unwrap();

        assert_eq!(
            r#"{"value":"100000.000","asset":"0000000000000000000000000000000000000000000000000000000000000000","ticker":"FOO"}"#,
            serialized
        );
    }

    #[test]
    fn new_balance_entry_no_precision_uses_0() {
        let usdt_ad = AssetDescription {
            asset_id: AssetId::default(),
            ticker: Some("FOO".to_string()),
            precision: None,
            status: Some(AssetStatus { confirmed: true }),
        };
        let entry = BalanceEntry::for_asset(100_000_000, usdt_ad, "L-BTC");

        let serialized = serde_json::to_string(&entry).unwrap();

        assert_eq!(
            r#"{"value":"100000000","asset":"0000000000000000000000000000000000000000000000000000000000000000","ticker":"FOO"}"#,
            serialized
        );
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod browser_tests {
    use wasm_bindgen_test::*;

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    pub async fn given_no_wallet_when_getting_address_then_fails() {
        let current_wallet = Mutex::default();

        let result = get_address("no-existent-wallet".to_owned(), &current_wallet).await;

        assert_eq!(
            result,
            Err(JsValue::from_str(
                "wallet with name 'no-existent-wallet' is currently not loaded"
            ))
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
        let result = get_address("wallet-2".to_owned(), &current_wallet).await;

        assert_eq!(
            result,
            Err(JsValue::from_str(
                "wallet with name 'wallet-2' is currently not loaded"
            ))
        );
    }

    #[wasm_bindgen_test]
    pub async fn cannot_create_two_wallets_with_same_name() {
        let current_wallet = Mutex::default();

        create_new("wallet-3".to_owned(), "foo".to_owned(), &current_wallet)
            .await
            .unwrap();
        let result = create_new("wallet-3".to_owned(), "foo".to_owned(), &current_wallet).await;

        assert_eq!(
            result,
            Err(JsValue::from_str(
                "wallet with name 'wallet-3' already exists"
            ))
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

        let result = load_existing("wallet-4".to_owned(), "foo".to_owned(), &current_wallet).await;

        assert_eq!(
            result,
            Err(JsValue::from_str(
                "cannot load wallet 'wallet-4' because wallet 'wallet-5' is currently loaded"
            ))
        );
    }

    #[wasm_bindgen_test]
    pub async fn cannot_load_wallet_with_wrong_password() {
        let current_wallet = Mutex::default();

        create_new("wallet-6".to_owned(), "foo".to_owned(), &current_wallet)
            .await
            .unwrap();
        unload_current(&current_wallet).await;

        let result = load_existing("wallet-6".to_owned(), "bar".to_owned(), &current_wallet).await;

        assert_eq!(
            result,
            Err(JsValue::from_str("bad password for wallet 'wallet-6'"))
        );
    }

    #[wasm_bindgen_test]
    pub async fn cannot_load_wallet_that_doesnt_exist() {
        let current_wallet = Mutex::default();

        let result = load_existing("foobar".to_owned(), "bar".to_owned(), &current_wallet).await;

        assert_eq!(
            result,
            Err(JsValue::from_str("wallet 'foobar' does not exist"))
        );
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
