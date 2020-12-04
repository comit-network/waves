use fixed_rate::fixed_rate;
use warp::Filter;

#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_methods(vec!["GET"])
        .allow_header("content-type");

    let latest_rate = warp::path("rate")
        .and(warp::get())
        .map(|| warp::sse::reply(warp::sse::keep_alive().stream(fixed_rate())));

    warp::serve(latest_rate.with(cors))
        .run(([127, 0, 0, 1], 3030))
        .await;
}

mod fixed_rate {
    use bobtimus::Rate;
    use futures::{stream, Stream};
    use std::{convert::Infallible, iter::repeat, time::Duration};
    use warp::sse::ServerSentEvent;

    pub fn fixed_rate() -> impl Stream<Item = Result<impl ServerSentEvent, Infallible>> {
        let data = Rate(1);
        let event = "rate";

        tokio::time::throttle(
            Duration::from_secs(5),
            stream::iter(
                repeat((event, data))
                    .map(|(event, data)| Ok((warp::sse::event(event), warp::sse::data(data)))),
            ),
        )
    }
}
