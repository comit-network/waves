use crate::{problem, Bobtimus, CreateSwapPayload, LatestRate, Rate};
use elements_fun::{
    encode::serialize_hex,
    secp256k1::rand::{CryptoRng, RngCore},
};
use futures::{Stream, StreamExt};
use rust_embed::RustEmbed;
use std::{convert::Infallible, sync::Arc};
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
    latest_rate_subscription: impl Stream<Item = Rate> + Clone + Send + Sync + 'static,
) -> BoxedFilter<(impl Reply,)>
where
    R: RngCore + CryptoRng + Clone + Send + Sync + 'static,
    RS: LatestRate + Clone + Send + Sync + 'static,
{
    let index_html = warp::path::end().and_then(serve_index);
    let waves_resources = warp::path::tail().and_then(serve_waves_resources);

    let latest_rate = warp::path!("api" / "rate" / "lbtc-lusdt")
        .and(warp::get())
        .map(move || latest_rate(latest_rate_subscription.clone()));

    let create_swap = warp::post()
        .and(warp::path!("api" / "swap" / "lbtc-lusdt" / "sell"))
        .and(warp::body::json())
        .and_then(move |payload| {
            let bobtimus = bobtimus.clone();
            async move {
                let mut bobtimus = bobtimus.lock().await;
                create_swap(&mut bobtimus, payload).await
            }
        });

    index_html
        .or(latest_rate)
        .or(create_swap)
        .or(waves_resources)
        .recover(problem::unpack_problem)
        .boxed()
}

async fn create_swap<R, RS>(
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
        .handle_create_swap(payload)
        .await
        .map(|transaction| serialize_hex(&transaction))
        .map_err(anyhow::Error::from)
        .map_err(problem::from_anyhow)
        .map_err(warp::reject::custom)
}

fn latest_rate<S>(stream: S) -> impl Reply
where
    S: Stream<Item = Rate> + Clone + Send + 'static,
{
    warp::sse::reply(warp::sse::keep_alive().stream(stream.map(|data| {
        Result::<_, Infallible>::Ok((warp::sse::event("rate"), warp::sse::json(data)))
    })))
}

async fn serve_index() -> Result<impl Reply, Rejection> {
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
