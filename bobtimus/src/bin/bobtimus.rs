use anyhow::Result;
use bobtimus::{
    cli::Config, database::Sqlite, elements_rpc::Client, http, kraken, liquidate_loans, Bobtimus,
};
use elements::{
    bitcoin::secp256k1::Secp256k1,
    secp256k1_zkp::rand::{rngs::StdRng, thread_rng, SeedableRng},
};
use std::{collections::HashMap, sync::Arc};
use tokio::{join, sync::Mutex};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    match Config::parse()? {
        Config::Start {
            elementsd_url,
            http,
            usdt_asset_id,
            db_file,
            https,
        } => {
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
                lender_states: HashMap::new(),
            };
            let bobtimus = Arc::new(Mutex::new(bobtimus));

            let https = https.map(|https| {
                warp::serve(http::routes(bobtimus.clone(), subscription.clone()))
                    .tls()
                    .cert_path(https.tls_certificate)
                    .key_path(https.tls_private_key)
                    .run(https.listen_https)
            });

            let http = http.map(|listen_http| {
                warp::serve(http::routes(bobtimus, subscription)).run(listen_http)
            });

            match (http, https) {
                (Some(http), Some(https)) => {
                    join!(http, https);
                }
                (Some(http), None) => {
                    http.await;
                }
                (None, Some(https)) => {
                    https.await;
                }
                _ => {}
            }
        }
        Config::LiquidateLoans {
            elementsd_url,
            db_file,
        } => {
            let db = Sqlite::new(db_file.as_path())?;
            let elementsd = Client::new(elementsd_url.into())?;

            liquidate_loans(&elementsd, db).await?;
        }
    }

    Ok(())
}
