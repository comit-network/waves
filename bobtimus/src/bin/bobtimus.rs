use anyhow::Result;
use bobtimus::{cli::StartCommand, routes, Bobtimus};
use elements_fun::secp256k1::rand::{rngs::StdRng, thread_rng, SeedableRng};
use elements_harness::Client;
use structopt::StructOpt;
use warp::Filter;

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

    let rate_service = kraken::RateService::new().await?;

    let bobtimus = Bobtimus {
        rng: StdRng::from_rng(&mut thread_rng()).unwrap(),
        rate_service,
        elementsd,
        btc_asset_id,
        usdt_asset_id,
    };

    let bobtimus_filter = warp::any().map({
        let bobtimus = bobtimus.clone();
        move || bobtimus.clone()
    });

    let subscription = bobtimus.rate_service.subscribe();
    let latest_rate = warp::path!("rate" / "lbtc-lusdt")
        .and(warp::get())
        .map(move || routes::latest_rate(subscription.clone()));

    let create_swap = warp::post()
        .and(warp::path!("swap" / "lbtc-lusdt"))
        .and(warp::path::end())
        .and(bobtimus_filter.clone())
        .and(warp::body::json())
        .and_then(routes::create_swap);

    warp::serve(latest_rate.or(create_swap))
        .run(([127, 0, 0, 1], api_port))
        .await;

    Ok(())
}

mod kraken {
    use anyhow::{anyhow, bail, Result};
    use async_trait::async_trait;
    use bobtimus::{LatestRate, LiquidUsdt, Rate};
    use futures::{SinkExt, Stream, StreamExt};
    use reqwest::Url;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::convert::TryFrom;
    use tokio::sync::watch;
    use tokio_tungstenite::tungstenite::Message;
    use watch::Receiver;

    const KRAKEN_WS_URL: &str = "wss://ws.kraken.com";
    const SUBSCRIBE_XBT_USDT_TICKER_PAYLOAD: &str = r#"
    { "event": "subscribe",
      "pair": [ "XBT/USD" ],
      "subscription": {
        "name": "ticker"
      }
    }"#;

    #[derive(Clone)]
    pub struct RateService(Receiver<Rate>);

    #[async_trait]
    impl LatestRate for RateService {
        async fn latest_rate(&mut self) -> anyhow::Result<Rate> {
            let rate = self
                .0
                .next()
                .await
                .ok_or_else(|| anyhow!("no latest rate"))?;

            Ok(rate)
        }
    }

    impl RateService {
        pub async fn new() -> Result<Self> {
            let (tx, rx) = watch::channel(bobtimus::Rate::default());

            let (ws, _response) =
                tokio_tungstenite::connect_async(Url::parse(KRAKEN_WS_URL).expect("valid url"))
                    .await?;

            let (mut write, mut read) = ws.split();

            tokio::spawn(async move {
                while let Some(msg) = read.next().await {
                    let msg = match msg {
                        Ok(Message::Text(msg)) => msg,
                        _ => continue,
                    };

                    let ticker = match serde_json::from_str::<TickerUpdate>(&msg) {
                        Ok(ticker) => ticker,
                        _ => continue,
                    };

                    let rate = match Rate::try_from(ticker) {
                        Ok(rate) => rate,
                        Err(e) => {
                            log::error!("could not get rate from ticker update: {}", e);
                            continue;
                        }
                    };

                    let _ = tx.broadcast(rate);
                }
            });

            write.send(SUBSCRIBE_XBT_USDT_TICKER_PAYLOAD.into()).await?;

            Ok(Self(rx))
        }

        pub fn subscribe(&self) -> impl Stream<Item = Rate> + Clone {
            self.0.clone()
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(transparent)]
    struct TickerUpdate(Vec<TickerField>);

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    enum TickerField {
        Data(TickerData),
        Metadata(Value),
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TickerData {
        #[serde(rename = "a")]
        ask: Vec<RateElement>,
        #[serde(rename = "b")]
        bid: Vec<RateElement>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    enum RateElement {
        Text(String),
        Number(u64),
    }

    impl TryFrom<TickerUpdate> for Rate {
        type Error = anyhow::Error;

        fn try_from(value: TickerUpdate) -> Result<Self> {
            let data = value
                .0
                .iter()
                .find_map(|field| match field {
                    TickerField::Data(data) => Some(data),
                    TickerField::Metadata(_) => None,
                })
                .ok_or_else(|| anyhow!("ticker update does not contain data"))?;

            let ask = data.ask.first().ok_or_else(|| anyhow!("no ask price"))?;
            let ask = match ask {
                RateElement::Text(ask) => LiquidUsdt::from_str_in_dollar(ask)?,
                _ => bail!("unexpected ask rate element"),
            };

            let bid = data.bid.first().ok_or_else(|| anyhow!("no bid price"))?;
            let bid = match bid {
                RateElement::Text(bid) => LiquidUsdt::from_str_in_dollar(bid)?,
                _ => bail!("unexpected bid rate element"),
            };

            Ok(Self { ask, bid })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn deserialize_ticker_update() {
            let sample_response = r#"
[2308,{"a":["18215.60000",0,"0.27454523"],"b":["18197.50000",0,"0.63711255"],"c":["18197.50000","0.00413060"],"v":["2.78915585","156.15766485"],"p":["18200.94036","18275.19149"],"t":[22,1561],"l":["18162.40000","17944.90000"],"h":["18220.90000","18482.60000"],"o":["18220.90000","18478.90000"]},"ticker","XBT/USDT"]"#;

            let _ = serde_json::from_str::<TickerUpdate>(sample_response).unwrap();
        }
    }
}
