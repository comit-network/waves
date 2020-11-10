use anyhow::Result;
use serde::Deserialize;

#[jsonrpc_client::api(version = "1.0")]
pub trait ElementsRpc {
    async fn getblockchaininfo(&self) -> BlockchainInfo;
}

#[jsonrpc_client::implement_async(ElementsRpc)]
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
}

#[derive(Debug, Deserialize)]
struct BlockchainInfo {
    pub chain: String,
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
            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let blockchain_info: BlockchainInfo = client.getblockchaininfo().await.unwrap();
        let network = blockchain_info.chain;

        assert_eq!(network, "elementsregtest")
    }
}
