use serde::{Deserialize, Serialize};
use wallet::{BalanceEntry, Trade};

#[derive(Debug, Deserialize, Serialize)]
/// Requests sent from the pop-up script to the background script.
pub enum ToBackground {
    UnlockRequest(String, String),
    CreateWalletRequest(String, String),
    BackgroundStatusRequest,
    BalanceRequest,
    SignRequest { tx_hex: String, tab_id: u32 },
}

#[derive(Debug, Deserialize, Serialize)]
/// Responses sent from the background script to the in-page script.
pub enum ToPopup {
    BalanceResponse(Vec<BalanceEntry>),
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
