use crate::Component;
use elements::Txid;
use rust_decimal::Decimal;
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
    GetBalance,
    GetSellCreateSwapPayload(String),
    GetBuyCreateSwapPayload(String),
    SignAndSend(String),
    WalletStatus(WalletStatus),
    Balance(Vec<BalanceEntry>),
    SellCreateSwapPayload(CreateSwapPayload),
    BuyCreateSwapPayload(CreateSwapPayload),
    SwapTxid(Txid),
    Hello(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BalanceEntry {
    pub asset: String,
    pub ticker: String,
    pub value: Decimal,
}
