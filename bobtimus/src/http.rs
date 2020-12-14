use elements_fun::secp256k1::rand::{CryptoRng, RngCore};
use futures::{Stream, StreamExt};
use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};

use crate::{Bobtimus, CreateSwapPayload, LatestRate, Rate};

pub async fn start<R, RS>(
    bobtimus: &Bobtimus<R, RS>,
    latest_rate_subscription: impl Stream<Item = Rate> + Clone + Send + Sync + 'static,
    port: u16,
) where
    R: RngCore + CryptoRng + Clone + Send + Sync + 'static,
    RS: LatestRate + Clone + Send + Sync + 'static,
{
    let latest_rate = warp::path!("rate" / "lbtc-lusdt")
        .and(warp::get())
        .map(move || latest_rate(latest_rate_subscription.clone()));

    let bobtimus_filter = warp::any().map({
        let bobtimus = bobtimus.clone();
        move || bobtimus.clone()
    });
    let create_swap = warp::post()
        .and(warp::path!("swap" / "lbtc-lusdt"))
        .and(warp::path::end())
        .and(bobtimus_filter)
        .and(warp::body::json())
        .and_then(create_swap);

    let routes = latest_rate.or(create_swap);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

async fn create_swap<R, RS>(
    mut bobtimus: Bobtimus<R, RS>,
    payload: CreateSwapPayload,
) -> Result<impl Reply, Rejection>
where
    R: RngCore + CryptoRng,
    RS: LatestRate,
{
    bobtimus
        .handle_create_swap(payload)
        .await
        .map(|transaction| warp::reply::json(&transaction))
        .map_err(|_| warp::reject::reject())
}

fn latest_rate<S>(stream: S) -> impl Reply
where
    S: Stream<Item = Rate> + Clone + Send + 'static,
{
    warp::sse::reply(warp::sse::keep_alive().stream(stream.map(|data| {
        Result::<_, Infallible>::Ok((warp::sse::event("rate"), warp::sse::json(data)))
    })))
}
