use crate::{ips_cs, Component};
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

impl From<Message> for ips_cs::Message {
    fn from(value: Message) -> Self {
        let rpc_data = match value.rpc_data {
            RpcData::GetWalletStatus => ips_cs::RpcData::GetWalletStatus,
            RpcData::GetSellCreateSwapPayload(payload) => {
                ips_cs::RpcData::GetSellCreateSwapPayload(payload)
            }
            RpcData::GetBuyCreateSwapPayload(payload) => {
                ips_cs::RpcData::GetBuyCreateSwapPayload(payload)
            }
            RpcData::SignAndSend(tx) => ips_cs::RpcData::SignAndSend(tx),
            RpcData::WalletStatus(status) => ips_cs::RpcData::WalletStatus(status),
            RpcData::SellCreateSwapPayload(payload) => {
                ips_cs::RpcData::SellCreateSwapPayload(payload)
            }
            RpcData::BuyCreateSwapPayload(payload) => {
                ips_cs::RpcData::BuyCreateSwapPayload(payload)
            }
            RpcData::SwapTxid(tx) => ips_cs::RpcData::SwapTxid(tx),
        };

        ips_cs::Message {
            rpc_data,
            target: Component::InPage,
            source: Component::Content,
        }
    }
}

impl From<ips_cs::Message> for Message {
    fn from(value: ips_cs::Message) -> Self {
        let rpc_data = match value.rpc_data {
            ips_cs::RpcData::GetWalletStatus => RpcData::GetWalletStatus,
            ips_cs::RpcData::GetSellCreateSwapPayload(payload) => {
                RpcData::GetSellCreateSwapPayload(payload)
            }
            ips_cs::RpcData::GetBuyCreateSwapPayload(payload) => {
                RpcData::GetBuyCreateSwapPayload(payload)
            }
            ips_cs::RpcData::SignAndSend(tx) => RpcData::SignAndSend(tx),
            ips_cs::RpcData::WalletStatus(status) => RpcData::WalletStatus(status),
            ips_cs::RpcData::SellCreateSwapPayload(payload) => {
                RpcData::SellCreateSwapPayload(payload)
            }
            ips_cs::RpcData::BuyCreateSwapPayload(payload) => {
                RpcData::BuyCreateSwapPayload(payload)
            }
            ips_cs::RpcData::SwapTxid(tx) => RpcData::SwapTxid(tx),
        };

        Message {
            rpc_data,
            target: Component::Background,
            source: Component::Content,
        }
    }
}
