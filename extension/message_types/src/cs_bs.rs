use crate::Component;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
    asset: String,
    ticker: String,
    value: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletStatus {
    pub loaded: bool,
    pub exists: bool,
}
