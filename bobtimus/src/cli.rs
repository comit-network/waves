use crate::USDT_ASSET_ID;
use elements::AssetId;
use reqwest::Url;

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
