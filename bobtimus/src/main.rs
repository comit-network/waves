#![deny(warnings)]
use warp::Filter;

#[tokio::main]
async fn main() {
    // GET /rate => 200 OK with body returning a random rate
    let routes = warp::path!("rate").map(|| {
        let rate = "4"; // thrown with a fair dice roll
        rate
    });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}