use crate::constants::{NATIVE_ASSET_ID, NATIVE_ASSET_TICKER, USDT_ASSET_ID};
use elements::AssetId;

pub fn lookup(asset_id: AssetId) -> Option<(&'static str, u8)> {
    if asset_id == *NATIVE_ASSET_ID {
        Some((NATIVE_ASSET_TICKER, 8))
    } else if asset_id == *USDT_ASSET_ID {
        Some(("USDt", 8))
    } else {
        None
    }
}
