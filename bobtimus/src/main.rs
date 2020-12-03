#![deny(warnings)]
use futures::{SinkExt, StreamExt};
use tokio::time::Duration;
use warp::{filters::ws::Message, Filter};

#[tokio::main]
async fn main() {
    let routes = warp::path("rate").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|websocket| async {
            let (mut send, _receive) = websocket.split();
            let mut round = 0;
            loop {
                round += 1;
                let rate = 4 + round % 10;
                let msg = Message::text(rate.to_string());
                let _result = send.send(msg).await;
                tokio::time::delay_for(Duration::from_secs(5)).await;
            }
        })
    });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
