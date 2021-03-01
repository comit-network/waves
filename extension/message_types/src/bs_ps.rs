use crate::Component;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Message to be send between background script and popup script
#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub rpc_data: RpcData,
    pub target: Component,
    pub source: Component,
    pub content_tab_id: u32,
}

// TODO: use proper types, this is just for ease of development
#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    pub value_map: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum RpcData {
    UnlockWallet(String, String),
    CreateWallet(String, String),
    GetWalletStatus,
    GetBalance,
    Balance(Vec<BalanceEntry>),
    Hello(String),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BalanceEntry {
    pub asset: String,
    pub ticker: String,
    pub value: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WalletStatus {
    None,
    NotLoaded,
    Loaded {
        balances: Vec<BalanceEntry>,
        address: String,
    },
}
