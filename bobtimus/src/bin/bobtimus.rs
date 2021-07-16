use anyhow::Result;
use bobtimus::{
    cli::Config, database::Sqlite, elements_rpc::Client, http, kraken, liquidate_loans,
    rendezvous::start_registration_loop, Bobtimus,
};
use elements::{
    bitcoin::secp256k1::Secp256k1,
    secp256k1_zkp::rand::{rngs::StdRng, thread_rng, SeedableRng},
};
use libp2p::{Multiaddr, PeerId};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    match Config::parse()? {
        Config::Start {
            elementsd_url,
            api_port,
            usdt_asset_id,
            db_file,
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

            // start libp2p behavior
            tokio::spawn(async move {
                let rendezvous_point_peer_id =
                    PeerId::from_str("12D3KooW9wRxMXDWz9KL3xNUTTy4YjuRSU3hcYk1iqZZ47vAZkgU")
                        .unwrap();
                let rendezvous_point_address =
                    Multiaddr::from_str("/ip4/127.0.0.1/tcp/8080").unwrap();
                let external_addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();
                let port = 9090;
                let _ = start_registration_loop(
                    rendezvous_point_peer_id,
                    rendezvous_point_address,
                    external_addr,
                    port,
                    "DEMO_NAMESPACE".to_string(),
                )
                .await
                .expect("To start registration loop");
            });

            // start http api
            warp::serve(http::routes(bobtimus, subscription))
                .run(([127, 0, 0, 1], api_port))
                .await;
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
