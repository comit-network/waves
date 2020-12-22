use anyhow::Result;
use bobtimus::{cli::StartCommand, http, kraken, Bobtimus};
use elements_fun::{
    bitcoin::secp256k1::Secp256k1,
    secp256k1::rand::{rngs::StdRng, thread_rng, SeedableRng},
};
use elements_harness::Client;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let StartCommand {
        elementsd_url,
        api_port,
        usdt_asset_id,
    } = StartCommand::from_args();

    let elementsd = Client::new(elementsd_url.into_string())?;
    let btc_asset_id = elementsd.get_bitcoin_asset_id().await?;

    let rate_service = kraken::RateService::new().await?;

    let bobtimus = Bobtimus {
        rng: StdRng::from_rng(&mut thread_rng()).unwrap(),
        rate_service,
        secp: Secp256k1::new(),
        elementsd,
        btc_asset_id,
        usdt_asset_id,
    };

    warp::serve(http::routes(&bobtimus, bobtimus.rate_service.subscribe()))
        .run(([127, 0, 0, 1], api_port))
        .await;

    Ok(())
}
