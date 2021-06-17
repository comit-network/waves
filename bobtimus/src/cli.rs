use crate::USDT_ASSET_ID;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use elements::AssetId;
use reqwest::Url;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(structopt::StructOpt, Debug)]
#[structopt(name = "bobtimus", about = "Auto-trader for L-BTC/L-USDt")]
pub struct StartCommand {
    #[structopt(default_value = "http://127.0.0.1:7042", long = "elementsd")]
    pub elementsd_url: Url,
    #[structopt(default_value = "3030")]
    pub api_port: u16,
    #[structopt(
        default_value = USDT_ASSET_ID,
        long = "usdt"
    )]
    pub usdt_asset_id: AssetId,
}

pub struct Config {
    pub elementsd_url: Url,
    pub api_port: u16,
    pub usdt_asset_id: AssetId,
}

impl Config {
    pub fn parse() -> Result<Self> {
        let StartCommand {
            elementsd_url,
            api_port,
            usdt_asset_id,
        } = StartCommand::from_args();

        Ok(Config {
            elementsd_url,
            api_port,
            usdt_asset_id,
        })
    }
}

/// This is the default location for the overall data-dir specific by system
///
/// Its default locations are platform specific: e.g.
/// Linux: /home/<user>/.local/share/project-waves/
/// OSX: /Users/<user>/Library/ApplicationSupport/project-waves/
fn system_data_dir() -> Result<PathBuf> {
    ProjectDirs::from("", "", "project-waves")
        .map(|proj_dirs| proj_dirs.data_dir().to_path_buf())
        .context("Could not generate default system data-dir dir path")
}
