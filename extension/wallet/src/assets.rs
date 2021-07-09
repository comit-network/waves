use crate::{BTC_ASSET_ID, USDT_ASSET_ID};
use elements::AssetId;
use wasm_bindgen::UnwrapThrowExt;

pub fn lookup(asset_id: AssetId) -> Option<(&'static str, u8)> {
    let bitcoin_asset_id = *BTC_ASSET_ID.lock().expect_throw("can get lock");
    let usdt_asset_id = *USDT_ASSET_ID.lock().expect_throw("can get lock");
    if asset_id == bitcoin_asset_id {
        Some(("L-BTC", 8))
    } else if asset_id == usdt_asset_id {
        Some(("L-USDt", 8))
    } else {
        None
    }
}
