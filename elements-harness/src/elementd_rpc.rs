use anyhow::{Context, Result};
use elements_fun::{
    bitcoin::Amount, bitcoin_hashes::hex::FromHex, encode::serialize_hex, Address, AssetId,
    Transaction, Txid,
};
use serde::Deserialize;
use std::collections::HashMap;

#[jsonrpc_client::api(version = "1.0")]
pub trait ElementsRpc {
    async fn getblockchaininfo(&self) -> BlockchainInfo;
    async fn getnewaddress(&self) -> Address;
    #[allow(clippy::too_many_arguments)]
    async fn sendtoaddress(
        &self,
        address: Address,
        amount: f64,
        comment: Option<String>,
        comment_to: Option<String>,
        subtract_fee_from_amount: Option<bool>,
        replaceable: Option<bool>,
        conf_target: Option<u64>,
        estimate_mode: Option<String>,
        asset_id: Option<AssetId>,
        ignore_blind_fail: bool,
    ) -> Txid;
    async fn dumpassetlabels(&self) -> HashMap<String, AssetId>;
    async fn getrawtransaction(&self, txid: Txid) -> String;
    async fn sendrawtransaction(&self, tx_hex: String) -> Txid;
    async fn issueasset(
        &self,
        asset_amount: f64,
        token_amount: f64,
        blind: bool,
    ) -> IssueAssetResponse;
    async fn getbalance(
        &self,
        dummy: Option<String>,
        minconf: Option<u64>,
        include_watchonly: Option<bool>,
        asset_id: Option<AssetId>,
    ) -> f64;
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
        let bitcoin_asset_id = labels
            .get(bitcoin_asset_tag)
            .context("failed to get asset id for bitcoin")?;

        Ok(*bitcoin_asset_id)
    }

    pub async fn send_asset_to_address(
        &self,
        address: Address,
        amount: Amount,
        asset_id: Option<AssetId>,
    ) -> Result<Txid> {
        let txid = self
            .sendtoaddress(
                address,
                amount.as_btc(),
                None,
                None,
                None,
                None,
                None,
                None,
                asset_id,
                true,
            )
            .await?;

        Ok(txid)
    }

    pub async fn get_raw_transaction(&self, txid: Txid) -> Result<Transaction> {
        let tx_hex = self.getrawtransaction(txid).await?;
        let tx = elements_fun::encode::deserialize(&Vec::<u8>::from_hex(&tx_hex).unwrap())?;

        Ok(tx)
    }

    pub async fn send_raw_transaction(&self, tx: &Transaction) -> Result<Txid> {
        let tx_hex = serialize_hex(tx);
        let txid = self.sendrawtransaction(tx_hex).await?;
        Ok(txid)
    }
}

#[derive(Debug, Deserialize)]
pub struct BlockchainInfo {
    pub chain: String,
    mediantime: u32,
}

#[derive(Debug, Deserialize)]
pub struct IssueAssetResponse {
    pub txid: Txid,
    pub vin: u8,
    pub entropy: String,
    pub asset: AssetId,
    pub token: String,
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
        let txid = client
            .sendtoaddress(address, 1.0, None, None, None, None, None, None, None, true)
            .await
            .unwrap();
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

    #[tokio::test]
    async fn issue_asset() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let expected_balance = 0.1;

        let asset_id = client
            .issueasset(expected_balance, 0.0, true)
            .await
            .unwrap()
            .asset;
        let balance = client
            .getbalance(None, None, None, Some(asset_id))
            .await
            .unwrap();

        let error_margin = f64::EPSILON;

        assert!((balance - expected_balance).abs() < error_margin)
    }
}
