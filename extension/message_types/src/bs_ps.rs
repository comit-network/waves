use crate::Component;
use serde::{Deserialize, Serialize};
use wallet::{BalanceEntry, Trade};

/// Message to be send between background script and popup script
#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub rpc_data: RpcData,
    pub target: Component,
    pub source: Component,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum RpcData {
    UnlockWallet(String, String),
    CreateWallet(String, String),
    GetWalletStatus,
    GetBalance,
    Balance(Vec<BalanceEntry>),
    SignAndSend { tx_hex: String, tab_id: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WalletStatus {
    None,
    NotLoaded,
    Loaded { address: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackgroundStatus {
    pub wallet: WalletStatus,
    pub sign_tx: Option<TransactionData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionData {
    pub hex: String,
    pub decoded: Trade,
    pub tab_id: u32,
}

impl BackgroundStatus {
    pub fn new(wallet: WalletStatus, sign_tx: Option<TransactionData>) -> Self {
        Self { wallet, sign_tx }
    }
}
