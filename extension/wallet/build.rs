use anyhow::{bail, Context, Result};
use elements::AssetId;
use std::{env, fs, path::Path};

// TODO: undo changes used in development mode
fn main() -> Result<()> {
    let out_dir = env::var_os("OUT_DIR").context("unable to access OUT_DIR")?;
    let constants_rs = Path::new(&out_dir).join("constants.rs");

    let native_asset_ticker = option_env!("NATIVE_ASSET_TICKER").unwrap_or("L-BTC");

    let native_asset_id = option_env!("NATIVE_ASSET_ID")
        .unwrap_or("5ac9f65c0efcc4775e0baec4ec03abdde22473cd3cf33c0419ca290e0751b225");
    let native_asset_id = native_asset_id
        .parse::<AssetId>()
        .with_context(|| format!("failed to parse {} as asset id", native_asset_id))?;

    let usdt_asset_id = option_env!("USDT_ASSET_ID")
        .unwrap_or("2dcf5a8834645654911964ec3602426fd3b9b4017554d3f9c19403e7fc1411d3");
    let usdt_asset_id = usdt_asset_id
        .parse::<AssetId>()
        .with_context(|| format!("failed to parse {} as asset id", usdt_asset_id))?;

    let esplora_api_url = option_env!("ESPLORA_API_URL")
        .as_deref()
        .unwrap_or("http://localhost:3012");

    let address_params = match option_env!("CHAIN") {
        Some("LIQUID") => "&elements::AddressParams::LIQUID",
        None | Some("ELEMENTS") => "&elements::AddressParams::ELEMENTS",
        Some(chain) => bail!("unsupported elements chain {}", chain),
    };

    let rate = option_env!("DEFAULT_SAT_PER_VBYTE")
        .as_deref()
        .unwrap_or("1.0");
    let rate = rate
        .parse::<f64>()
        .with_context(|| format!("failed to parse '{}' as f64", rate))?;

    fs::write(
        &constants_rs,
        &format!(
            r#"
use conquer_once::Lazy;

pub const NATIVE_ASSET_TICKER: &str = "{}";
pub static NATIVE_ASSET_ID: Lazy<elements::AssetId> = Lazy::new(|| elements::AssetId::from_slice(&{:?}).unwrap());
pub static USDT_ASSET_ID: Lazy<elements::AssetId> = Lazy::new(|| elements::AssetId::from_slice(&{:?}).unwrap());
pub const ESPLORA_API_URL: &str = "{}";
pub const ADDRESS_PARAMS: &elements::AddressParams = {};
pub const DEFAULT_SAT_PER_VBYTE: f32 = {:.4};
"#,
            native_asset_ticker,
            native_asset_id.into_inner().0,
            usdt_asset_id.into_inner().0,
            esplora_api_url,
            address_params,
            rate
        ),
    )
    .context("failed to write constants.rs file")?;

    Ok(())
}
