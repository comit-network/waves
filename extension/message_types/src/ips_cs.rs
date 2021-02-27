use crate::{
    cs_bs::{BalanceEntry, WalletStatus},
    Component,
};
use serde::{Deserialize, Serialize};

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
    Balance(Vec<BalanceEntry>),
    WalletStatus(WalletStatus),
    Hello(String),
}
