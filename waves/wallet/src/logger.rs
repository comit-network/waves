use crate::storage::Storage;
use anyhow::{Context, Result};

pub fn try_init() -> Result<()> {
    let local_storage = Storage::local_storage()?;

    let log_level = local_storage
        .get_item::<log::Level>("wallet_log")
        .context("failed to get `wallet_log` log level")?;

    if let Some(level) = log_level {
        wasm_logger::init(wasm_logger::Config::new(level));
    }

    Ok(())
}
