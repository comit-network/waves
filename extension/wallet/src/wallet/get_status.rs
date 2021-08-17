use crate::{
    storage::Storage,
    wallet::{ListOfWallets, Wallet},
};
use anyhow::Result;
use futures::lock::Mutex;

pub async fn get_status(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<WalletStatus> {
    let storage = Storage::local_storage()?;

    let wallets = storage
        .get_item::<ListOfWallets>("wallets")?
        .unwrap_or_default();
    let exists = wallets.has(&name);

    if !exists {
        return Ok(WalletStatus::None);
    }

    let guard = current_wallet.lock().await;

    let status = match &*guard {
        Some(wallet) if wallet.name == name => WalletStatus::Loaded {
            address: wallet.get_address(),
        },
        _ => WalletStatus::NotLoaded,
    };

    Ok(status)
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "status")]
pub enum WalletStatus {
    None,
    Loaded { address: elements::Address },
    NotLoaded,
}
