#![deny(warnings)]
use warp::Filter;

#[tokio::main]
async fn main() {
    let cors = warp::cors().allow_any_origin();

    // GET /rate => 200 OK with body returning a random rate
    let routes = warp::path!("rate")
        .map(|| {
            let rate = "4"; // thrown with a fair dice roll
            rate
        })
        .with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
