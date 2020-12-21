use anyhow::Result;
use bobtimus::{cli::StartCommand, fixed_rate, http, Bobtimus};
use elements_fun::{
    bitcoin::{secp256k1::Secp256k1, Amount},
    secp256k1::rand::{rngs::StdRng, thread_rng, SeedableRng},
    Address,
};
use elements_harness::{elementd_rpc::ElementsRpc, Client};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use warp::{Filter, Rejection, Reply};

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

    let rate_service = fixed_rate::Service::new();

    let bobtimus = Bobtimus {
        rng: StdRng::from_rng(&mut thread_rng()).unwrap(),
        rate_service,
        secp: Secp256k1::new(),
        elementsd,
        btc_asset_id,
        usdt_asset_id,
    };

    let routes = http::routes(&bobtimus, bobtimus.rate_service.subscribe());

    let bobtimus_filter = warp::any().map({
        let bobtimus = bobtimus.clone();
        move || bobtimus.clone()
    });
    let lusdt_faucet = warp::post()
        .and(warp::path!("faucet" / "lusdt"))
        .and(warp::path::end())
        .and(bobtimus_filter)
        .and(warp::body::json())
        .and_then(lusdt_faucet);

    warp::serve(routes.or(lusdt_faucet))
        .run(([127, 0, 0, 1], api_port))
        .await;

    Ok(())
}

async fn lusdt_faucet<RS>(
    bobtimus: Bobtimus<StdRng, RS>,
    payload: LusdtFaucetPayload,
) -> Result<impl Reply, Rejection> {
    let txid = bobtimus
        .elementsd
        .send_asset_to_address(
            payload.address,
            payload.amount,
            Some(bobtimus.usdt_asset_id),
        )
        .await
        .map_err(|e| {
            log::error!("could not fund address: {}", e);
            warp::reject::reject()
        })?;

    let address = bobtimus.elementsd.getnewaddress().await.map_err(|e| {
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

    Ok(warp::reply::json(&txid))
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct LusdtFaucetPayload {
    address: Address,
    #[serde(with = "::elements_fun::bitcoin::util::amount::serde::as_sat")]
    amount: Amount,
}
