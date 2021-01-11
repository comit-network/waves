use anyhow::{bail, Context, Result};
use elements_fun::secp256k1::SecretKey;
use futures::lock::Mutex;

use crate::{
    storage::Storage,
    wallet::{ListOfWallets, Wallet},
};

pub async fn create_new(
    name: String,
    password: String,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<()> {
    let storage = Storage::local_storage()?;

    let mut wallets = storage
        .get_item::<ListOfWallets>("wallets")?
        .unwrap_or_default();

    if wallets.has(&name) {
        bail!("wallet with name '{}' already exists", name);
    }

    let params = if cfg!(debug_assertions) {
        // use weak parameters in debug mode, otherwise this is awfully slow
        log::warn!("using extremely weak scrypt parameters for password hashing");
        scrypt::ScryptParams::new(1, 1, 1).unwrap()
    } else {
        scrypt::ScryptParams::recommended()
    };

    let hashed_password =
        scrypt::scrypt_simple(&password, &params).context("failed to hash password")?;

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
