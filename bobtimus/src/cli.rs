use crate::USDT_ASSET_ID;
use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use elements::AssetId;
use reqwest::Url;
use std::{net::SocketAddr, path::PathBuf};
use structopt::StructOpt;

#[derive(structopt::StructOpt, Debug)]
#[structopt(name = "bobtimus", about = "Auto-trader for L-BTC/L-USDt")]
pub enum Command {
    Start {
        #[structopt(default_value = "http://127.0.0.1:7042", long = "elementsd")]
        elementsd_url: Url,
        #[structopt(default_value = USDT_ASSET_ID, long = "usdt")]
        usdt_asset_id: AssetId,
        #[structopt(long, parse(from_os_str))]
        db_file: Option<PathBuf>,

        #[structopt(long = "http")]
        listen_http: Option<SocketAddr>,
        #[structopt(long = "https")]
        listen_https: Option<SocketAddr>,
        #[structopt(long, parse(from_os_str))]
        tls_certificate: Option<PathBuf>,
        #[structopt(long, parse(from_os_str))]
        tls_private_key: Option<PathBuf>,
    },
    LiquidateLoans {
        #[structopt(default_value = "http://127.0.0.1:7042", long = "elementsd")]
        elementsd_url: Url,
        #[structopt(long, parse(from_os_str))]
        db_file: Option<PathBuf>,
    },
}

pub struct Https {
    pub listen_https: SocketAddr,
    pub tls_certificate: PathBuf,
    pub tls_private_key: PathBuf,
}

pub enum Config {
    Start {
        elementsd_url: Url,
        usdt_asset_id: AssetId,
        db_file: PathBuf,
        http: Option<SocketAddr>,
        https: Option<Https>,
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
                listen_http,
                listen_https,
                usdt_asset_id,
                db_file,
                tls_certificate,
                tls_private_key,
            } => {
                if listen_http.is_none() && listen_https.is_none() {
                    bail!("Neither listening on HTTP nor HTTPS were configured, this server is pointless");
                }

                let https = match (listen_https, tls_certificate, tls_private_key) {
                    (Some(listen_https), Some(tls_certificate), Some(tls_private_key)) => {
                        Some(Https {
                            listen_https,
                            tls_certificate,
                            tls_private_key,
                        })
                    }
                    (None, None, None) => None,
                    _ => bail!(
                        "HTTPS has to be configured with TLS server private key and certificate"
                    ),
                };

                Config::Start {
                    elementsd_url,
                    http: listen_http,
                    usdt_asset_id,
                    db_file: resolve_db_file(db_file)?,
                    https,
                }
            }
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
