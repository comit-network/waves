use crate::wallet::{current, Wallet};
use anyhow::Result;
use elements_fun::Address;
use futures::lock::Mutex;

pub async fn get_address(name: String, current_wallet: &Mutex<Option<Wallet>>) -> Result<Address> {
    let wallet = current(&name, current_wallet).await?;

    let address = wallet.get_address();

    Ok(address)
}
