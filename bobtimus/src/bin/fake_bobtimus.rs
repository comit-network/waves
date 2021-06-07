use anyhow::Result;
use bobtimus::{cli::StartCommand, fixed_rate, http, Bobtimus, LiquidUsdt};
use elements::{
    bitcoin::{secp256k1::Secp256k1, Amount},
    secp256k1_zkp::rand::{rngs::StdRng, thread_rng, SeedableRng},
    Address,
};
use elements_harness::{elementd_rpc::ElementsRpc, Client};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::sync::Mutex;
use warp::{Filter, Rejection, Reply};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let StartCommand {
        elementsd_url,
        api_port,
        usdt_asset_id,
    } = StartCommand::from_args();

    let elementsd = Client::new(elementsd_url.into())?;
    let btc_asset_id = elementsd.get_bitcoin_asset_id().await?;

    let rate_service = fixed_rate::Service::new();
    let subscription = rate_service.subscribe();

    let bobtimus = Bobtimus {
        rng: StdRng::from_rng(&mut thread_rng()).unwrap(),
        rate_service,
        secp: Secp256k1::new(),
        elementsd,
        btc_asset_id,
        usdt_asset_id,
    };
    let bobtimus = Arc::new(Mutex::new(bobtimus));

    let routes = http::routes(bobtimus.clone(), subscription);

    let cors = warp::cors().allow_any_origin();

    let faucet = warp::post()
        .and(warp::path!("api" / "faucet" / Address))
        .and_then(move |address| {
            let bobtimus = bobtimus.clone();
            async move {
                let mut bobtimus = bobtimus.lock().await;
                faucet(&mut bobtimus, address).await
            }
        });

    warp::serve(routes.or(faucet).with(cors))
        .run(([127, 0, 0, 1], api_port))
        .await;

    Ok(())
}

async fn faucet<R, RS>(
    bobtimus: &mut Bobtimus<R, RS>,
    address: Address,
) -> Result<impl Reply, Rejection> {
    let mut txids = Vec::new();
    for (asset_id, amount) in &[
        (bobtimus.btc_asset_id, Amount::from_sat(1_000_000_000)),
        (
            bobtimus.usdt_asset_id,
            LiquidUsdt::from_str_in_dollar("200000.0")
                .expect("valid dollars")
                .into(),
        ),
    ] {
        let txid = bobtimus
            .elementsd
            .send_asset_to_address(&address, *amount, Some(*asset_id))
            .await
            .map_err(|e| {
                log::error!(
                    "could not fund address {} with asset {}: {}",
                    address,
                    asset_id,
                    e
                );
                warp::reject::reject()
            })?;

        txids.push(txid);
    }

    let _ = bobtimus
        .elementsd
        .reissueasset(bobtimus.usdt_asset_id, 200000.0)
        .await
        .map_err(|e| {
            log::error!("could not reissue asset: {}", e);
            warp::reject::reject()
        })?;

    let address = bobtimus
        .elementsd
        .get_new_address(None)
        .await
        .map_err(|e| {
            log::error!("could not get new address: {}", e);
            warp::reject::reject()
        })?;
    bobtimus
        .elementsd
        .generatetoaddress(1, &address)
        .await
        .map_err(|e| {
            log::error!("could not generate block: {}", e);
            warp::reject::reject()
        })?;

    Ok(warp::reply::json(&txids))
}
