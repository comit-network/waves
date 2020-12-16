use anyhow::Result;
use bobtimus::{cli::StartCommand, fixed_rate, http, Bobtimus};
use elements_fun::{
    bitcoin::secp256k1::Secp256k1,
    secp256k1::rand::{rngs::StdRng, thread_rng, SeedableRng},
};
use elements_harness::Client;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<()> {
    let StartCommand {
        elementsd_url,
        api_port,
        usdt_asset_id,
    } = StartCommand::from_args();

    let elementsd = Client::new(elementsd_url.into_string())?;
    let btc_asset_id = elementsd.get_bitcoin_asset_id().await?;

    let rate_service = fixed_rate::Service::new();

    let bobtimus = Bobtimus {
        rng: StdRng::from_rng(&mut thread_rng()).unwrap(),
        rate_service,
        secp: Secp256k1::new(),
        elementsd,
        btc_asset_id,
        usdt_asset_id,
    };

    http::start(&bobtimus, bobtimus.rate_service.subscribe(), api_port).await;

    Ok(())
}
