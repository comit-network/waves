use crate::{problem, Bobtimus, CreateSwapPayload, LatestRate, RateSubscription};
use anyhow::Context;
use elements::{
    encode::{deserialize, serialize_hex},
    secp256k1_zkp::rand::{CryptoRng, RngCore},
};
use futures::{StreamExt, TryStreamExt};
use rust_embed::RustEmbed;
use std::{error::Error, fmt, sync::Arc};
use tokio::sync::Mutex;
use warp::{
    filters::BoxedFilter, http::header::HeaderValue, path::Tail, reply::Response, Filter,
    Rejection, Reply,
};

#[derive(RustEmbed)]
#[folder = "../waves/dist/"]
struct Waves;

pub fn routes<R, RS>(
    bobtimus: Arc<Mutex<Bobtimus<R, RS>>>,
    latest_rate_subscription: RateSubscription,
) -> BoxedFilter<(impl Reply,)>
where
    R: RngCore + CryptoRng + Clone + Send + Sync + 'static,
    RS: LatestRate + Clone + Send + Sync + 'static,
{
    let index_html = warp::get().and(warp::path::tail()).and_then(serve_index);
    let waves_resources = warp::get()
        .and(warp::path("app"))
        .and(warp::path::tail())
        .and_then(serve_waves_resources);

    let latest_rate = warp::get()
        .and(warp::path!("api" / "rate" / "lbtc-lusdt"))
        .map(move || latest_rate(latest_rate_subscription.clone()));

    let create_buy_swap = warp::post()
        .and(warp::path!("api" / "swap" / "lbtc-lusdt" / "buy"))
        .and(warp::body::json())
        .and_then({
            let bobtimus = bobtimus.clone();
            move |payload| {
                let bobtimus = bobtimus.clone();
                async move {
                    let mut bobtimus = bobtimus.lock().await;
                    create_buy_swap(&mut bobtimus, payload).await
                }
            }
        });

    let create_sell_swap = warp::post()
        .and(warp::path!("api" / "swap" / "lbtc-lusdt" / "sell"))
        .and(warp::body::json())
        .and_then({
            let bobtimus = bobtimus.clone();
            move |payload| {
                let bobtimus = bobtimus.clone();
                async move {
                    let mut bobtimus = bobtimus.lock().await;
                    create_sell_swap(&mut bobtimus, payload).await
                }
            }
        });

    let create_loan = warp::post()
        .and(warp::path!("api" / "loan" / "lbtc-lusdt"))
        .and(warp::body::json())
        .and_then({
            let bobtimus = bobtimus.clone();
            move |payload| {
                let bobtimus = bobtimus.clone();
                async move {
                    let mut bobtimus = bobtimus.lock().await;
                    create_loan(&mut bobtimus, payload).await
                }
            }
        });

    let finalize_loan = warp::post()
        .and(warp::path!("api" / "loan" / "lbtc-lusdt" / "finalize"))
        .and(warp::body::json())
        .and_then(move |payload| {
            let bobtimus = bobtimus.clone();
            async move {
                let mut bobtimus = bobtimus.lock().await;
                finalize_loan(&mut bobtimus, payload)
                    .await
                    .map_err(anyhow::Error::from)
                    .map_err(problem::from_anyhow)
                    .map_err(warp::reject::custom)
            }
        });

    latest_rate
        .or(create_sell_swap)
        .or(create_buy_swap)
        .or(create_loan)
        .or(finalize_loan)
        .or(waves_resources)
        .or(index_html)
        .recover(problem::unpack_problem)
        .boxed()
}

async fn create_buy_swap<R, RS>(
    bobtimus: &mut Bobtimus<R, RS>,
    payload: serde_json::Value,
) -> Result<impl Reply, Rejection>
where
    R: RngCore + CryptoRng,
    RS: LatestRate,
{
    let payload = payload.to_string();
    let payload: CreateSwapPayload = serde_json::from_str(&payload)
        .map_err(anyhow::Error::from)
        .map_err(problem::from_anyhow)
        .map_err(warp::reject::custom)?;

    bobtimus
        .handle_create_buy_swap(payload)
        .await
        .map(|transaction| serialize_hex(&transaction))
        .map_err(anyhow::Error::from)
        .map_err(problem::from_anyhow)
        .map_err(warp::reject::custom)
}

async fn create_sell_swap<R, RS>(
    bobtimus: &mut Bobtimus<R, RS>,
    payload: serde_json::Value,
) -> Result<impl Reply, Rejection>
where
    R: RngCore + CryptoRng,
    RS: LatestRate,
{
    let payload = payload.to_string();
    let payload: CreateSwapPayload = serde_json::from_str(&payload)
        .map_err(anyhow::Error::from)
        .map_err(problem::from_anyhow)
        .map_err(warp::reject::custom)?;

    bobtimus
        .handle_create_sell_swap(payload)
        .await
        .map(|transaction| serialize_hex(&transaction))
        .map_err(anyhow::Error::from)
        .map_err(problem::from_anyhow)
        .map_err(warp::reject::custom)
}

async fn create_loan<R, RS>(
    bobtimus: &mut Bobtimus<R, RS>,
    payload: serde_json::Value,
) -> Result<impl Reply, Rejection>
where
    R: RngCore + CryptoRng,
    RS: LatestRate,
{
    let payload = payload.to_string();
    let payload = serde_json::from_str(&payload)
        .map_err(anyhow::Error::from)
        .map_err(problem::from_anyhow)
        .map_err(warp::reject::custom)?;

    bobtimus
        .handle_loan_request(payload)
        .await
        .map(|loan_response| warp::reply::json(&loan_response))
        .map_err(anyhow::Error::from)
        .map_err(problem::from_anyhow)
        .map_err(warp::reject::custom)
}

#[derive(serde::Deserialize)]
struct FinalizeLoanPayload {
    tx_hex: String,
}

async fn finalize_loan<R, RS>(
    bobtimus: &mut Bobtimus<R, RS>,
    payload: serde_json::Value,
) -> anyhow::Result<impl Reply>
where
    R: RngCore + CryptoRng,
    RS: LatestRate,
{
    let payload: FinalizeLoanPayload = serde_json::from_value(payload)?;

    let payload = deserialize(&hex::decode(&payload.tx_hex)?)?;

    bobtimus
        .finalize_loan(payload)
        .await
        .map(|loan_response| warp::reply::json(&loan_response))
}

fn latest_rate(subscription: RateSubscription) -> impl Reply {
    let stream = subscription
        .into_stream()
        .map_ok(|data| {
            let event = warp::sse::Event::default()
                .id("rate")
                .json_data(data)
                .context("failed to attach json data to sse event")?;

            Ok(event)
        })
        .map(|result| match result {
            Ok(Ok(ok)) => Ok(ok),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(e),
        })
        .err_into::<RateStreamError>();

    warp::sse::reply(warp::sse::keep_alive().stream(stream))
}

#[derive(Debug)]
struct RateStreamError(anyhow::Error);

impl fmt::Display for RateStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#}", self.0)
    }
}

impl std::error::Error for RateStreamError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl From<anyhow::Error> for RateStreamError {
    fn from(e: anyhow::Error) -> Self {
        RateStreamError(e)
    }
}

async fn serve_index(_path: Tail) -> Result<impl Reply, Rejection> {
    serve_impl("index.html")
}

async fn serve_waves_resources(path: Tail) -> Result<impl Reply, Rejection> {
    serve_impl(path.as_str())
}

fn serve_impl(path: &str) -> Result<impl Reply, Rejection> {
    let asset = Waves::get(path).ok_or_else(warp::reject::not_found)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut res = Response::new(asset.into());
    res.headers_mut().insert(
        "content-type",
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(res)
}
