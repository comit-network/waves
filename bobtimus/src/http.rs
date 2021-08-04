use crate::{problem, Bobtimus, LatestRate, RateSubscription};
use anyhow::Context;
use elements::{
    encode::serialize_hex,
    secp256k1_zkp::rand::{thread_rng, CryptoRng, RngCore},
    Transaction,
};
use futures::{StreamExt, TryStreamExt};
use rust_embed::RustEmbed;
use std::{error::Error, fmt, sync::Arc};
use tokio::sync::Mutex;
use warp::{
    filters::BoxedFilter,
    http::{header::HeaderValue, HeaderMap},
    path::Tail,
    reply::Response,
    Filter, Rejection, Reply,
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

    // This header is needed so that SSE works through a proxy
    let mut sse_headers = HeaderMap::new();
    sse_headers.insert("Cache-Control", HeaderValue::from_static("no-transform"));

    let latest_rate = warp::get()
        .and(warp::path!("api" / "rate" / "lbtc-lusdt"))
        .map(move || latest_rate(latest_rate_subscription.clone()))
        .with(warp::reply::with::headers(sse_headers));

    let create_buy_swap = warp::post()
        .and(warp::path!("api" / "swap" / "lbtc-lusdt" / "buy"))
        .and(warp::body::json())
        .and_then({
            let bobtimus = bobtimus.clone();
            move |payload| {
                let bobtimus = bobtimus.clone();
                async move {
                    bobtimus
                        .lock()
                        .await
                        .handle_create_buy_swap(payload)
                        .await
                        .map(|transaction| serialize_hex(&transaction))
                        .map_err(anyhow::Error::from)
                        .map_err(problem::from_anyhow)
                        .map_err(warp::reject::custom)
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
                    bobtimus
                        .lock()
                        .await
                        .handle_create_sell_swap(payload)
                        .await
                        .map(|transaction| serialize_hex(&transaction))
                        .map_err(anyhow::Error::from)
                        .map_err(problem::from_anyhow)
                        .map_err(warp::reject::custom)
                }
            }
        });

    let offer_loan = warp::get()
        .and(warp::path!("api" / "loan" / "lbtc-lusdt"))
        .and_then({
            let bobtimus = bobtimus.clone();
            move || {
                let bobtimus = bobtimus.clone();
                async move {
                    bobtimus
                        .lock()
                        .await
                        .handle_loan_offer_request()
                        .await
                        .map(|loan_offer| warp::reply::json(&loan_offer))
                        .map_err(anyhow::Error::from)
                        .map_err(problem::from_anyhow)
                        .map_err(warp::reject::custom)
                }
            }
        });

    let take_loan = warp::post()
        .and(warp::path!("api" / "loan" / "lbtc-lusdt"))
        .and(warp::body::json())
        .and_then({
            let bobtimus = bobtimus.clone();
            move |payload| {
                let bobtimus = bobtimus.clone();

                async move {
                    bobtimus
                        .lock()
                        .await
                        .handle_loan_request(payload)
                        .await
                        .map(|loan_response| warp::reply::json(&loan_response))
                        .map_err(anyhow::Error::from)
                        .map_err(problem::from_anyhow)
                        .map_err(warp::reject::custom)
                }
            }
        });

    let finalize_loan = warp::post()
        .and(warp::path!("api" / "loan" / "lbtc-lusdt" / "finalize"))
        .and(warp::body::json())
        .and_then(move |payload: FinalizeLoanPayload| {
            let bobtimus = bobtimus.clone();
            async move {
                bobtimus
                    .lock()
                    .await
                    .finalize_loan(payload.tx_hex)
                    .await
                    .map(|loan_response| warp::reply::json(&loan_response))
                    .map_err(anyhow::Error::from)
                    .map_err(problem::from_anyhow)
                    .map_err(warp::reject::custom)
            }
        });

    latest_rate
        .or(create_sell_swap)
        .or(create_buy_swap)
        .or(offer_loan)
        .or(take_loan)
        .or(finalize_loan)
        .or(waves_resources)
        .or(index_html)
        .recover(problem::unpack_problem)
        .boxed()
}

#[derive(serde::Deserialize)]
struct FinalizeLoanPayload {
    #[serde(with = "baru::loan::transaction_as_string")]
    tx_hex: Transaction,
}

fn latest_rate(subscription: RateSubscription) -> impl Reply {
    let stream = subscription
        .into_stream()
        .map_ok(|data| {
            let event = warp::sse::Event::default()
                .id(thread_rng().next_u32().to_string())
                .event("rate")
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
