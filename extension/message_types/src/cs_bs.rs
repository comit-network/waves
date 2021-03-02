use crate::Component;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use wallet::WalletStatus;

/// Message to be send between content script and background script
#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub rpc_data: RpcData,
    pub target: Component,
    pub source: Component,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum RpcData {
    GetWalletStatus,
    GetBalance,
    Balance(Vec<BalanceEntry>),
    WalletStatus(WalletStatus),
    Hello(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BalanceEntry {
    pub asset: String,
    pub ticker: String,
    pub value: Decimal,
}
