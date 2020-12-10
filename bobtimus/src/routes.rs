use elements_fun::secp256k1::rand::{CryptoRng, RngCore};
use futures::{Stream, StreamExt};
use std::convert::Infallible;
use warp::{Rejection, Reply};

use crate::{Bobtimus, CreateSwapPayload, LatestRate, Rate};

pub async fn create_swap<R, RS>(
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

pub fn latest_rate<S>(stream: S) -> impl Reply
where
    S: Stream<Item = Rate> + Clone + Send + 'static,
{
    warp::sse::reply(warp::sse::keep_alive().stream(stream.map(|data| {
        Result::<_, Infallible>::Ok((warp::sse::event("rate"), warp::sse::json(data)))
    })))
}
