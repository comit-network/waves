use crate::cache_storage::CacheStorage;
use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use baru::GetUtxos;
use elements::{
    encode::{deserialize, serialize_hex},
    Address, BlockHash, OutPoint, Transaction, TxOut, Txid,
};
use futures::{stream::FuturesUnordered, TryStreamExt};
use reqwest::{StatusCode, Url};

#[derive(Clone)]
pub struct EsploraClient {
    base_url: Url,
}

impl EsploraClient {
    pub fn new(base_url: Url) -> Self {
        Self { base_url }
    }

    pub async fn fetch_transaction(&self, txid: Txid) -> Result<Transaction> {
        fetch_transaction(self.base_url.clone(), txid).await
    }

    pub async fn broadcast(&self, tx: Transaction) -> Result<Txid> {
        let esplora_url = self.base_url.clone();
        let esplora_url = esplora_url.join("tx")?;
        let client = reqwest::Client::new();

        let response = client
            .post(esplora_url.clone())
            .body(serialize_hex(&tx))
            .send()
            .await?;

        let code = response.status();

        if !code.is_success() {
            bail!("failed to successfully publish transaction");
        }

        let txid = response
            .text()
            .await?
            .parse()
            .context("failed to parse response body as txid")?;

        Ok(txid)
    }

    /// Fetch transaction history for the specified address.
    ///
    /// Returns up to 50 mempool transactions plus the first 25 confirmed
    /// transactions. See
    /// https://github.com/blockstream/esplora/blob/master/API.md#get-addressaddresstxs
    /// for more information.
    pub async fn fetch_transaction_history(&self, address: &Address) -> Result<Vec<Txid>> {
        let path = format!("address/{}/txs", address);
        let base_url = self.base_url.clone();
        let url = base_url.join(path.as_str())?;
        let response = reqwest::get(url.clone())
            .await
            .context("failed to fetch transaction history")?;

        if !response.status().is_success() {
            let error_body = response.text().await?;
            return Err(anyhow!(
                "failed to fetch transaction history, esplora returned '{}' from '{}'",
                error_body,
                url
            ));
        }

        #[derive(serde::Deserialize)]
        struct HistoryElement {
            txid: Txid,
        }

        let response = response
            .json::<Vec<HistoryElement>>()
            .await
            .context("failed to deserialize response")?;

        Ok(response.iter().map(|elem| elem.txid).collect())
    }

    pub async fn get_fee_estimates(&self) -> Result<FeeEstimatesResponse> {
        let base_url = self.base_url.clone();
        let esplora_url = base_url.join("fee-estimates")?;

        let fee_estimates = reqwest::get(esplora_url.clone())
            .await
            .with_context(|| format!("failed to GET {}", esplora_url))?
            .json()
            .await
            .context("failed to deserialize fee estimates")?;

        Ok(fee_estimates)
    }

    pub async fn get_block_height(&self) -> Result<u32> {
        let base_url = self.base_url.clone();
        let esplora_url = base_url.join("/blocks/tip/height")?;

        let latest_block_height = reqwest::get(esplora_url.clone())
            .await
            .with_context(|| format!("failed to GET {}", esplora_url))?
            .json()
            .await
            .context("failed to deserialize latest block height")?;

        Ok(latest_block_height)
    }
}

#[async_trait(?Send)]
impl GetUtxos for EsploraClient {
    async fn get_utxos(&self, address: Address) -> Result<Vec<(OutPoint, TxOut)>> {
        let base_url = self.base_url.clone();
        let utxos = fetch_utxos(base_url.clone(), address).await?;

        let txouts = utxos
            .into_iter()
            .map(move |utxo| {
                let base_url = base_url.clone();
                async move {
                    let mut tx = fetch_transaction(base_url, utxo.txid).await?;
                    let txout = tx.output.remove(utxo.vout as usize);
                    let utxo = OutPoint {
                        txid: utxo.txid,
                        vout: utxo.vout,
                    };
                    Result::<_, anyhow::Error>::Ok((utxo, txout))
                }
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        Ok(txouts)
    }
}

/// Fetch the UTXOs of an address.
///
/// UTXOs change over time and as such, this function never uses a cache.
async fn fetch_utxos(base_url: Url, address: Address) -> Result<Vec<Utxo>> {
    let path = format!("address/{}/utxo", address);
    let esplora_url = base_url.join(path.as_str())?;
    let response = reqwest::get(esplora_url.clone())
        .await
        .context("failed to fetch UTXOs")?;

    if response.status() == StatusCode::NOT_FOUND {
        log::debug!(
            "GET {} returned 404, defaulting to empty UTXO set",
            esplora_url
        );

        return Ok(Vec::new());
    }

    if !response.status().is_success() {
        let error_body = response.text().await?;
        return Err(anyhow!(
            "failed to fetch utxos, esplora returned '{}'",
            error_body
        ));
    }

    let mut utxos = response
        .json::<Vec<Utxo>>()
        .await
        .context("failed to deserialize response")?;

    // Sort UTXOs to have more deterministic output in case something goes wrong.
    // Note that the order of these UTXOs does not have to be strictly assured.
    utxos.sort_by(|l, r| l.txid.cmp(&r.txid).then(l.vout.cmp(&r.vout)));

    Ok(utxos)
}

/// Fetches a transaction.
///
/// This function makes use of the browsers local storage to avoid spamming the underlying source.
/// Transaction never change after they've been mined, hence we can cache those indefinitely.
pub async fn fetch_transaction(base_url: Url, txid: Txid) -> Result<Transaction> {
    let cache = CacheStorage::new()?;
    let body = cache
        .match_or_add(&format!("{}/tx/{}/hex", base_url, txid))
        .await?
        .text()
        .await?;

    Ok(deserialize(&hex::decode(body.as_bytes())?)?)
}

/// The response object for the `/fee-estimates` endpoint.
///
/// The key is the confirmation target (in number of blocks) and the value is the estimated feerate (in sat/vB).
/// The available confirmation targets are 1-25, 144, 504 and 1008 blocks.
#[derive(serde::Deserialize, Debug)]
pub struct FeeEstimatesResponse {
    #[serde(rename = "1")]
    pub b_1: Option<f32>,
    #[serde(rename = "2")]
    pub b_2: Option<f32>,
    #[serde(rename = "3")]
    pub b_3: Option<f32>,
    #[serde(rename = "4")]
    pub b_4: Option<f32>,
    #[serde(rename = "5")]
    pub b_5: Option<f32>,
    #[serde(rename = "6")]
    pub b_6: Option<f32>,
    #[serde(rename = "7")]
    pub b_7: Option<f32>,
    #[serde(rename = "8")]
    pub b_8: Option<f32>,
    #[serde(rename = "9")]
    pub b_9: Option<f32>,
    #[serde(rename = "10")]
    pub b_10: Option<f32>,
    #[serde(rename = "11")]
    pub b_11: Option<f32>,
    #[serde(rename = "12")]
    pub b_12: Option<f32>,
    #[serde(rename = "13")]
    pub b_13: Option<f32>,
    #[serde(rename = "14")]
    pub b_14: Option<f32>,
    #[serde(rename = "15")]
    pub b_15: Option<f32>,
    #[serde(rename = "16")]
    pub b_16: Option<f32>,
    #[serde(rename = "17")]
    pub b_17: Option<f32>,
    #[serde(rename = "18")]
    pub b_18: Option<f32>,
    #[serde(rename = "19")]
    pub b_19: Option<f32>,
    #[serde(rename = "20")]
    pub b_20: Option<f32>,
    #[serde(rename = "21")]
    pub b_21: Option<f32>,
    #[serde(rename = "22")]
    pub b_22: Option<f32>,
    #[serde(rename = "23")]
    pub b_23: Option<f32>,
    #[serde(rename = "24")]
    pub b_24: Option<f32>,
    #[serde(rename = "25")]
    pub b_25: Option<f32>,
    #[serde(rename = "144")]
    pub b_144: Option<f32>,
    #[serde(rename = "504")]
    pub b_504: Option<f32>,
    #[serde(rename = "1008")]
    pub b_1008: Option<f32>,
}

/// Represents a UTXO as it is modeled by esplora.
///
/// We ignore the commitments and asset IDs because we need to fetch the full transaction anyway.
/// Hence, we don't even bother with deserializing it here.
#[derive(serde::Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct Utxo {
    pub txid: Txid,
    pub vout: u32,
    pub status: UtxoStatus,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct UtxoStatus {
    pub confirmed: bool,
    pub block_height: Option<u64>,
    pub block_hash: Option<BlockHash>,
    pub block_time: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_deserialize_confidential_utxo() {
        let utxos = r#"[
  {
    "txid": "26ad78aca6db29fa6ca37337fcfb23498dc1a01ee274614970097ab7ca6b6a19",
    "vout": 0,
    "status": {
      "confirmed": true,
      "block_height": 1099688,
      "block_hash": "e0dd686b1a3334e941512a0e08dda69c9db71cd642d8b219f6063fb81838d86b",
      "block_time": 1607042939
    },
    "valuecommitment": "0959edffa4326a255a15925a5a8eeda37e27fb80a62b1f1792dcd98bb8e29b7496",
    "assetcommitment": "0b7b0f23047a44d6145fb4754f218807c1a3f0acc811221f7ba35e44dfc3a31795",
    "noncecommitment": "039b1feace0413efc144298bc462a90bbf8f269cf68e3dfa65088f84f381921261"
  }
]
"#;

        let utxos = serde_json::from_str::<Vec<Utxo>>(utxos).unwrap();

        assert_eq!(utxos.len(), 1);
    }

    #[test]
    fn can_deserialize_explicit_utxo() {
        let utxos = r#"[
  {
    "txid": "58035633e6391fd08955f9f73b710efe3835a7975baaf1267aa4fcb3c738c1ba",
    "vout": 0,
    "status": {
      "confirmed": true,
      "block_height": 1099644,
      "block_hash": "58d573591f8920b225512bb209b5d75f2ae9260f107c306b87a53c4cc4d42d7e",
      "block_time": 1607040059
    },
    "value": 99958,
    "asset": "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d"
  }
]
"#;

        let utxos = serde_json::from_str::<Vec<Utxo>>(utxos).unwrap();

        assert_eq!(utxos.len(), 1);
    }
}
