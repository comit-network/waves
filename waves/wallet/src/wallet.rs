use crate::{
    esplora,
    esplora::{AssetDescription, Utxo},
    storage::Storage,
    SECP,
};
use aes_gcm_siv::{
    aead::{Aead, NewAead},
    Aes256GcmSiv,
};
use anyhow::Context;
use elements_fun::{
    bitcoin,
    bitcoin::secp256k1::SecretKey,
    secp256k1::{rand, PublicKey},
    Address, AddressParams, AssetId, TxOut,
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
use wasm_bindgen::{JsValue, UnwrapThrowExt};

pub async fn create_new(
    name: String,
    password: String,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<(), JsValue> {
    let storage = Storage::local_storage()?;

    let mut wallets = storage
        .get_item::<ListOfWallets>("wallets")?
        .unwrap_or_default();

    if wallets.has(&name) {
        return Err(JsValue::from_str(&format!(
            "wallet with name '{}' already exists",
            name
        )));
    }

    let params = if cfg!(debug_assertions) {
        // use weak parameters in debug mode, otherwise this is awfully slow
        log::warn!("using extremely weak scrypt parameters for password hashing");
        scrypt::ScryptParams::new(1, 1, 1).unwrap()
    } else {
        scrypt::ScryptParams::recommended()
    };

    let hashed_password = map_err_from_anyhow!(
        scrypt::scrypt_simple(&password, &params).context("failed to hash password")
    )?;

    let new_wallet = Wallet::initialize_new(
        name.clone(),
        password,
        SecretKey::new(&mut rand::thread_rng()),
    )?;

    storage.set_item(&format!("wallets.{}.password", name), hashed_password)?;
    storage.set_item(
        &format!("wallets.{}.secret_key", name),
        format!(
            "{}${}",
            hex::encode(new_wallet.sk_salt),
            hex::encode(new_wallet.encrypted_secret_key()?)
        ),
    )?;
    wallets.add(name);
    storage.set_item("wallets", wallets)?;

    current_wallet.lock().await.replace(new_wallet);

    log::info!("New wallet successfully initialized");

    Ok(())
}

pub async fn load_existing(
    name: String,
    password: String,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<(), JsValue> {
    let mut guard = current_wallet.lock().await;

    if let Some(Wallet { name: loaded, .. }) = &*guard {
        return Err(JsValue::from_str(&format!(
            "cannot load wallet '{}' because wallet '{}' is currently loaded",
            name, loaded
        )));
    }

    let storage = Storage::local_storage()?;
    let wallets = storage
        .get_item::<ListOfWallets>("wallets")?
        .unwrap_or_default();

    if !wallets.has(&name) {
        return Err(JsValue::from_str(&format!(
            "wallet '{}' does not exist",
            name
        )));
    }

    let stored_password = storage
        .get_item::<String>(&format!("wallets.{}.password", name))?
        .ok_or_else(|| JsValue::from_str("no password stored for wallet"))?;

    scrypt::scrypt_check(&password, &stored_password)
        .map_err(|_| JsValue::from_str(&format!("bad password for wallet '{}'", name)))?;

    let sk_ciphertext = storage
        .get_item::<String>(&format!("wallets.{}.secret_key", name))?
        .ok_or_else(|| JsValue::from_str("no secret key for wallet"))?;

    let wallet = Wallet::initialize_existing(name, password, sk_ciphertext)?;

    guard.replace(wallet);

    log::info!("Wallet successfully loaded");

    Ok(())
}

pub async fn unload_current(current_wallet: &Mutex<Option<Wallet>>) {
    let mut guard = current_wallet.lock().await;

    if guard.is_none() {
        log::debug!("Wallet is already unloaded");
        return;
    }

    *guard = None;
}

pub async fn get_status(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<WalletStatus, JsValue> {
    let storage = Storage::local_storage()?;

    let wallets = storage
        .get_item::<ListOfWallets>("wallets")?
        .unwrap_or_default();
    let exists = wallets.has(&name);

    let guard = current_wallet.lock().await;
    let loaded = guard.as_ref().map_or(false, |w| w.name == name);

    Ok(WalletStatus { loaded, exists })
}

pub async fn get_address(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<Address, JsValue> {
    let wallet = current(&name, current_wallet).await?;

    let address = wallet.get_address()?;

    Ok(address)
}

pub async fn get_balances(
    name: &str,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<Vec<BalanceEntry>, JsValue> {
    let wallet = current(name, current_wallet).await?;

    let txouts = get_txouts(&wallet).await?;

    let grouped_txouts = txouts
        .into_iter()
        .filter_map(|utxo| match utxo {
            TxOut::Explicit(explicit) => Some((explicit.asset.0, explicit.value.0)),
            TxOut::Confidential(confidential) => {
                match confidential.unblind(&*SECP, wallet.blinding_key()) {
                    Ok(unblinded_txout) => Some((unblinded_txout.asset, unblinded_txout.value)),
                    Err(e) => {
                        log::warn!("failed to unblind txout: {}", e);
                        None
                    }
                }
            }
            TxOut::Null(_) => None,
        })
        .group_by(|(asset, _)| *asset);

    let balances = (&grouped_txouts)
        .into_iter()
        .map(|(asset, utxos)| async move {
            let ad = match esplora::fetch_asset_description(&asset).await {
                Ok(ad) => ad,
                Err(e) => {
                    log::debug!("failed to fetched asset description: {}", e);

                    AssetDescription::default(asset)
                }
            };
            let total_sum = utxos.map(|(_, value)| value).sum();

            let native_asset_ticker = option_env!("NATIVE_ASSET_TICKER").unwrap_or("L-BTC");

            BalanceEntry::for_asset(total_sum, ad, native_asset_ticker)
        })
        .collect::<FuturesUnordered<_>>()
        .collect()
        .await;

    Ok(balances)
}

pub async fn current<'n, 'w>(
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

async fn get_txouts(wallet: &Wallet) -> Result<Vec<TxOut>, JsValue> {
    let address = wallet.get_address()?;

    let utxos = map_err_from_anyhow!(esplora::fetch_utxos(&address).await)?;

    let txouts = map_err_from_anyhow!(
        utxos
            .into_iter()
            .map(|Utxo { txid, vout, .. }| async move {
                let tx = esplora::fetch_transaction(txid).await;

                tx.map(|mut tx| tx.output.remove(vout as usize))
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await
    )?;

    Ok(txouts)
}

/// A single balance entry as returned by [`get_balances`].
#[derive(Debug, serde::Serialize)]
pub struct BalanceEntry {
    value: Decimal,
    asset: AssetId,
    /// The ticker symbol of the asset.
    ///
    /// Not all assets are part of the registry and as such, not all of them have a ticker symbol.
    ticker: Option<String>,
}

impl BalanceEntry {
    /// Construct a new [`BalanceEntry`] using the given value and [`AssetDescription`].
    ///
    /// [`AssetDescriptions`] are different for native vs user-issued assets. To properly handle these cases, we pass in the `native_asset_ticker` that should be used in case we are handling a native asset.
    pub fn for_asset(
        value: u64,
        asset: AssetDescription,
        native_asset_ticker: &'static str,
    ) -> Self {
        let precision = if asset.is_native_asset() {
            8
        } else {
            asset.precision.unwrap_or(0)
        };

        let mut decimal = Decimal::from(value);
        decimal
            .set_scale(precision)
            .expect("precision must be < 28");

        let ticker = if asset.is_native_asset() {
            Some(native_asset_ticker.to_owned())
        } else {
            asset.ticker
        };

        Self {
            value: decimal,
            asset: asset.asset_id,
            ticker,
        }
    }
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

    pub fn get_address(&self) -> Result<Address, JsValue> {
        let public_key = PublicKey::from_secret_key(&*SECP, &self.secret_key);
        let blinding_key = PublicKey::from_secret_key(&*SECP, &self.blinding_key());

        let address = Address::p2wpkh(
            &bitcoin::PublicKey {
                compressed: true,
                key: public_key,
            },
            Some(blinding_key),
            &AddressParams::ELEMENTS,
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

#[derive(Debug, serde::Serialize)]
pub struct WalletStatus {
    loaded: bool,
    exists: bool,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::esplora::AssetStatus;
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

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
    use super::*;
    use wasm_bindgen_test::*;
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
