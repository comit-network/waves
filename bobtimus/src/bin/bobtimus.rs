use anyhow::Result;
use bobtimus::{cli::Config, database::Sqlite, http, kraken, Bobtimus};
use elements::{
    bitcoin::secp256k1::Secp256k1,
    secp256k1::rand::{rngs::StdRng, thread_rng, SeedableRng},
};
use elements_harness::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let Config {
        elementsd_url,
        api_port,
        usdt_asset_id,
        db_file,
    } = Config::parse()?;
    let db = Sqlite::new(db_file.as_path())?;

    let elementsd = Client::new(elementsd_url.into())?;
    let btc_asset_id = elementsd.get_bitcoin_asset_id().await?;

    let rate_service = kraken::RateService::new().await?;
    let subscription = rate_service.subscribe();

    let bobtimus = Bobtimus {
        rng: StdRng::from_rng(&mut thread_rng()).unwrap(),
        rate_service,
        secp: Secp256k1::new(),
        elementsd,
        btc_asset_id,
        usdt_asset_id,
        db,
    };
    let bobtimus = Arc::new(Mutex::new(bobtimus));

    warp::serve(http::routes(bobtimus, subscription))
        .run(([127, 0, 0, 1], api_port))
        .await;

    Ok(())
}
