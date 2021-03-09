use crate::Component;
use elements::Txid;
use serde::{Deserialize, Serialize};
use wallet::{CreateSwapPayload, WalletStatus};

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
    GetSellCreateSwapPayload(String),
    GetBuyCreateSwapPayload(String),
    SignAndSend(String),
    WalletStatus(WalletStatus),
    SellCreateSwapPayload(CreateSwapPayload),
    BuyCreateSwapPayload(CreateSwapPayload),
    SwapTxid(Txid),
}
