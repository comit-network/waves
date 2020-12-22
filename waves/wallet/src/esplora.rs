use crate::cache_storage::CacheStorage;
use anyhow::{anyhow, bail, Context, Result};
use conquer_once::Lazy;
use elements_fun::{
    encode::{deserialize, serialize_hex},
    Address, AssetId, BlockHash, Transaction, Txid,
};
use reqwest::StatusCode;

static ELEMENTS_ESPLORA_URL: Lazy<&str> = Lazy::new(|| {
    option_env!("ESPLORA_URL")
        .as_deref()
        .unwrap_or("https://blockstream.info/liquid/api")
});

/// Fetch the UTXOs of an address.
///
/// UTXOs change over time and as such, this function never uses a cache.
pub async fn fetch_utxos(address: &Address) -> Result<Vec<Utxo>> {
    let url = format!("{}/address/{}/utxo", ELEMENTS_ESPLORA_URL, address);
    let response = reqwest::get(&url).await.context("failed to fetch UTXOs")?;

    if response.status() == StatusCode::NOT_FOUND {
        log::debug!("GET {} returned 404, defaulting to empty UTXO set", url);

        return Ok(Vec::new());
    }

    if !response.status().is_success() {
        let error_body = response.text().await?;
        return Err(anyhow!(
            "failed to fetch utxos, esplora returned '{}'",
            error_body
        ));
    }

    response
        .json::<Vec<Utxo>>()
        .await
        .context("failed to deserialize response")
}

pub async fn fetch_asset_description(asset: AssetId) -> Result<AssetDescription> {
    let cache = CacheStorage::from_window()?
        .open("asset_descriptions")
        .await?;

    let asset_description = cache
        .match_or_add(&format!("{}/asset/{}", ELEMENTS_ESPLORA_URL, asset))
        .await?
        .json()
        .await?;

    Ok(asset_description)
}

/// Fetches a transaction.
///
/// This function makes use of the browsers cache to avoid spamming the underlying source.
/// Transaction never change after they've been mined, hence we can cache those indefinitely.
pub async fn fetch_transaction(txid: Txid) -> Result<Transaction> {
    let cache = CacheStorage::from_window()?.open("transactions").await?;

    let body = cache
        .match_or_add(&format!("{}/tx/{}/hex", ELEMENTS_ESPLORA_URL, txid))
        .await?
        .text()
        .await?;

    Ok(deserialize(&hex::decode(body.as_bytes())?)?)
}

pub async fn broadcast(tx: Transaction) -> Result<Txid> {
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/tx", ELEMENTS_ESPLORA_URL))
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

pub async fn get_fee_estimates() -> Result<FeeEstimates> {
    let url = &format!("{}/fee-estimates", ELEMENTS_ESPLORA_URL);

    let fee_estimates = reqwest::get(url)
        .await
        .with_context(|| format!("failed to GET {}", url))?
        .json()
        .await
        .context("failed to deserialize fee estimates")?;

    Ok(fee_estimates)
}

#[derive(serde::Deserialize, Debug)]
pub struct FeeEstimates {
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
    pub block_height: u64,
    pub block_hash: BlockHash,
    pub block_time: u64,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct AssetDescription {
    pub asset_id: AssetId,
    pub ticker: Option<String>,
    pub precision: Option<u32>,
    pub status: Option<AssetStatus>,
}

impl AssetDescription {
    /// Checks if the given asset is a native asset.
    ///
    /// Native assets don't have a `status` field among many others. We could also test for any of the other fields but testing for `status` seems to be the most sane way because the native asset is `confirmed` from the very beginning and not from a particular block.
    pub fn is_native_asset(&self) -> bool {
        self.status.is_none()
    }

    pub const fn default(asset_id: AssetId) -> Self {
        Self {
            asset_id,
            ticker: None,
            precision: None,
            status: None,
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct AssetStatus {
    pub confirmed: bool,
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

    #[test]
    fn can_deserialize_asset_description() {
        let desc = r#"{
  "asset_id": "4d372612132098147e94ad40688c51b733c19c88c7c2d3a6bb3e4ee5b67002e2",
  "issuance_txin": {
    "txid": "14d1e51e23bf5fe770b26e286b3502d36a478a4830800c895b414775ecdcdc57",
    "vin": 0
  },
  "issuance_prevout": {
    "txid": "e41a96edc72187953b41bb10c6aa899220d814ad5c9a6f28ecbba711b960180c",
    "vout": 3
  },
  "reissuance_token": "3f36d33a0de64402e21db6ba7aeccc1829ab1873998ae3b3bd5bd8e36814b154",
  "contract_hash": "b99b5498972014cf0273c3734d443e695d831a8d89917bb4cc4b19a85f58874f",
  "status": {
    "confirmed": true,
    "block_height": 794293,
    "block_hash": "75b1da0e75b7209d6156a45318f7462f6232b4475d7bbdeffa0f7356b01fd18c",
    "block_time": 1588286398
  },
  "chain_stats": {
    "tx_count": 1,
    "issuance_count": 1,
    "issued_amount": 100000,
    "burned_amount": 0,
    "has_blinded_issuances": false,
    "reissuance_tokens": 1000,
    "burned_reissuance_tokens": 0
  },
  "mempool_stats": {
    "tx_count": 0,
    "issuance_count": 0,
    "issued_amount": 0,
    "burned_amount": 0,
    "has_blinded_issuances": false,
    "reissuance_tokens": null,
    "burned_reissuance_tokens": 0
  },
  "contract": {
    "entity": {
      "domain": "explorer.lightnite.io"
    },
    "issuer_pubkey": "02a1d99dc9e8cd006e24230aa5d7a088c92d617ef33f52d71b5eb089bc9c0f35e4",
    "name": "Alien Bag",
    "precision": 0,
    "ticker": "AAlBa",
    "version": 0
  },
  "entity": {
    "domain": "explorer.lightnite.io"
  },
  "precision": 0,
  "name": "Alien Bag",
  "ticker": "AAlBa"
}"#;

        let _desc = serde_json::from_str::<AssetDescription>(desc).unwrap();
    }
}
