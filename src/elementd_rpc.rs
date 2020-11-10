use crate::json_rpc;
use anyhow::Result;
use reqwest::Url;
use serde::Deserialize;

pub const JSONRPC_VERSION: &str = "1.0";

#[derive(Debug, Clone)]
pub struct Client {
    rpc_client: json_rpc::Client,
}

impl Client {
    pub fn new(url: Url) -> Self {
        Client {
            rpc_client: json_rpc::Client::new(url),
        }
    }

    pub async fn network(&self) -> Result<String> {
        let blockchain_info = self.blockchain_info().await?;

        Ok(blockchain_info.chain)
    }

    async fn blockchain_info(&self) -> Result<BlockchainInfo> {
        let blockchain_info = self
            .rpc_client
            .send::<Vec<()>, BlockchainInfo>(json_rpc::Request::new(
                "getblockchaininfo",
                vec![],
                JSONRPC_VERSION.into(),
            ))
            .await?;

        Ok(blockchain_info)
    }
}

#[derive(Debug, Deserialize)]
struct BlockchainInfo {
    chain: String,
    mediantime: u32,
}

#[cfg(all(test))]
mod test {
    use super::*;
    use crate::Elementsd;
    use testcontainers::clients;

    #[tokio::test]
    async fn get_network_info() {
        let tc_client = clients::Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (Client::new(blockchain.node_url.clone()), blockchain)
        };

        let network = client.network().await.unwrap();

        assert_eq!(network, "regtest")
    }
}
