use crate::constants::{NATIVE_ASSET_ID, NATIVE_ASSET_TICKER, USDT_ASSET_ID};
use elements_fun::AssetId;

pub const fn lookup(asset_id: AssetId) -> Option<(&'static str, u8)> {
    Some(match asset_id {
        NATIVE_ASSET_ID => (NATIVE_ASSET_TICKER, 8),
        USDT_ASSET_ID => ("USDt", 8),
        _ => return None,
    })
}
