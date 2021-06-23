use crate::USDT_ASSET_ID;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use elements::AssetId;
use reqwest::Url;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(structopt::StructOpt, Debug)]
#[structopt(name = "bobtimus", about = "Auto-trader for L-BTC/L-USDt")]
pub enum Command {
    Start {
        #[structopt(default_value = "http://127.0.0.1:7042", long = "elementsd")]
        elementsd_url: Url,
        #[structopt(default_value = "3030")]
        api_port: u16,
        #[structopt(
        default_value = USDT_ASSET_ID,
        long = "usdt"
    )]
        usdt_asset_id: AssetId,
        #[structopt(short, parse(from_os_str))]
        db_file: Option<PathBuf>,
    },
    LiquidateLoans {
        #[structopt(default_value = "http://127.0.0.1:7042", long = "elementsd")]
        elementsd_url: Url,
        #[structopt(short, parse(from_os_str))]
        db_file: Option<PathBuf>,
    },
}

pub enum Config {
    Start {
        elementsd_url: Url,
        api_port: u16,
        usdt_asset_id: AssetId,
        db_file: PathBuf,
    },
    LiquidateLoans {
        elementsd_url: Url,
        db_file: PathBuf,
    },
}

impl Config {
    pub fn parse() -> Result<Self> {
        let config = match Command::from_args() {
            Command::Start {
                elementsd_url,
                api_port,
                usdt_asset_id,
                db_file,
            } => Config::Start {
                elementsd_url,
                api_port,
                usdt_asset_id,
                db_file: resolve_db_file(db_file)?,
            },
            Command::LiquidateLoans {
                elementsd_url,
                db_file,
            } => Config::LiquidateLoans {
                elementsd_url,
                db_file: resolve_db_file(db_file)?,
            },
        };

        Ok(config)
    }
}

fn resolve_db_file(db_file: Option<PathBuf>) -> Result<PathBuf, anyhow::Error> {
    Ok(match db_file {
        None => {
            let path_buf = system_data_dir()?.join("bobtimus.sql");
            tracing::info!(
                "DB file not provided. Falling back to default path at {}",
                path_buf.display()
            );
            path_buf
        }
        Some(db_file) => db_file,
    })
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
