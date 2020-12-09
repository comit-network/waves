use crate::cache_storage::CacheStorage;
use anyhow::{Context, Result};
use conquer_once::Lazy;
use elements_fun::{encode::deserialize, Address, BlockHash, Transaction, Txid};
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_futures::JsFuture;

static LIQUID_ESPLORA_URL: Lazy<&str> = Lazy::new(|| {
    option_env!("ESPLORA_URL")
        .as_deref()
        .unwrap_or_else(|| "https://blockstream.info/liquid")
});

/// Fetch the UTXOs of an address.
///
/// UTXOs change over time and as such, this function never uses a cache.
pub async fn fetch_utxos(address: &Address) -> Result<Vec<Utxo>> {
    reqwest::get(&format!(
        "{}/api/address/{}/utxo",
        LIQUID_ESPLORA_URL, address
    ))
    .await
    .context("failed to fetch UTXOs")?
    .json::<Vec<Utxo>>()
    .await
    .context("failed to deserialize response")
}

/// Fetches a transaction.
///
/// This function makes use of the browsers cache to avoid spamming the underlying source.
/// Transaction never change after they've been mined, hence we can cache those indefinitely.
pub async fn fetch_transaction(txid: Txid) -> Result<Transaction> {
    let window = web_sys::window().unwrap_throw();

    let storage = CacheStorage::from(try_anyhow!(window.caches())?);
    let cache = try_anyhow!(storage.open("transactions").await)?;

    let url = &format!("{}/api/tx/{}/hex", LIQUID_ESPLORA_URL, txid);

    let response = match try_anyhow!(cache.match_with_str(url).await)? {
        Some(response) => response,
        None => {
            try_anyhow!(cache.add_with_str(url).await)?;

            // we just put it in the cache, it is gotta be there
            // TODO: if the request failed with a 400, it will not be there :)
            try_anyhow!(cache.match_with_str(url).await)?.context("no response in cache")?
        }
    };

    let body = try_anyhow!(JsFuture::from(try_anyhow!(response.text())?).await)?
        .as_string()
        .context("response is not a string")?;

    Ok(deserialize(&hex::decode(body.as_bytes())?)?)
}

/// Represents a UTXO as it is modeled by esplora.
///
/// We ignore the commitments and asset IDs because we need to fetch the full transaction anyway.
/// Hence, we don't even bother with deserializing it here.
#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct Utxo {
    pub txid: Txid,
    pub vout: u32,
    pub status: UtxoStatus,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct UtxoStatus {
    pub confirmed: bool,
    pub block_height: u64,
    pub block_hash: BlockHash,
    pub block_time: u64,
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
