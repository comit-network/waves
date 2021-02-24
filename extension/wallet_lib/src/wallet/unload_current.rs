use crate::wallet::Wallet;
use futures::lock::Mutex;

pub async fn unload_current(current_wallet: &Mutex<Option<Wallet>>) {
    let mut guard = current_wallet.lock().await;

    if guard.is_none() {
        log::debug!("Wallet is already unloaded");
        return;
    }

    *guard = None;
}
