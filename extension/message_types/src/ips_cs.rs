use crate::{cs_bs::BalanceEntry, Component};
use serde::{Deserialize, Serialize};
use wallet::{CreateSwapPayload, WalletStatus};

/// Message to be send between in-page script and content script
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub rpc_data: RpcData,
    pub target: Component,
    pub source: Component,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum RpcData {
    GetWalletStatus,
    GetBalance,
    GetSellCreateSwapPayload(String),
    WalletStatus(WalletStatus),
    Balance(Vec<BalanceEntry>),
    SellCreateSwapPayload(CreateSwapPayload),
    Hello(String),
}
