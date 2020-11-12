use anyhow::Result;
use anyhow::Context;
use elements::{bitcoin::Txid, Address, AssetId};
use serde::Deserialize;
use std::collections::HashMap;

#[jsonrpc_client::api(version = "1.0")]
pub trait ElementsRpc {
    async fn getblockchaininfo(&self) -> BlockchainInfo;
    async fn getnewaddress(&self) -> Address;
    async fn sendtoaddress(&self, address: Address, amount: f64) -> Txid;
    async fn dumpassetlabels(&self) -> HashMap<String, AssetId>;
    async fn getrawtransaction(&self, txid: Txid) -> String;
    async fn sendrawtransaction(&self, tx_hex: String) -> Txid;
}

#[jsonrpc_client::implement(ElementsRpc)]
#[derive(Debug)]
pub struct Client {
    inner: reqwest::Client,
    base_url: reqwest::Url,
}

impl Client {
    pub fn new(base_url: String) -> Result<Self> {
        Ok(Self {
            inner: reqwest::Client::new(),
            base_url: base_url.parse()?,
        })
    }

    pub async fn get_bitcoin_asset_id(&self) -> Result<AssetId> {
        let labels = self.dumpassetlabels().await?;
        let bitcoin_asset_tag = "bitcoin";
        let bitcoin_asset_id = labels.get(bitcoin_asset_tag).context("failed to get asset id for bitcoin")?;

        Ok(bitcoin_asset_id.clone())
    }
}

#[derive(Debug, Deserialize)]
pub struct BlockchainInfo {
    pub chain: String,
    mediantime: u32,
}

#[cfg(all(test))]
mod test {
    use super::*;
    use crate::Elementsd;
    use testcontainers::clients::Cli;

    #[tokio::test]
    async fn get_network_info() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();
            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let blockchain_info: BlockchainInfo = client.getblockchaininfo().await.unwrap();
        let network = blockchain_info.chain;

        assert_eq!(network, "elementsregtest")
    }

    #[tokio::test]
    async fn send_to_generated_address() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let address = client.getnewaddress().await.unwrap();
        let txid = client.sendtoaddress(address, 1.0).await.unwrap();
        let _tx_hex = client.getrawtransaction(txid).await.unwrap();
    }

    #[tokio::test]
    async fn dump_labels() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let _labels = client.dumpassetlabels().await.unwrap();
    }
}
