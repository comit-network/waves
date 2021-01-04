use crate::{LatestRate, LiquidUsdt, Rate, RateSubscription};
use std::{convert::TryFrom, time::Duration};
use tokio::{
    sync::watch::{self, Receiver},
    time::sleep,
};

#[derive(Clone)]
pub struct Service(Receiver<Rate>);

impl Service {
    pub fn new() -> Self {
        let data = fixed_rate();
        let (tx, rx) = watch::channel(data);

        tokio::spawn(async move {
            loop {
                let _ = tx.send(data);

                sleep(Duration::from_secs(5)).await;
            }
        });

        Self(rx)
    }

    pub fn subscribe(&self) -> RateSubscription {
        RateSubscription::from(self.0.clone())
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}

impl LatestRate for Service {
    fn latest_rate(&mut self) -> Rate {
        fixed_rate()
    }
}

fn fixed_rate() -> Rate {
    Rate {
        ask: LiquidUsdt::try_from(20_000.0).unwrap(),
        bid: LiquidUsdt::try_from(19_000.0).unwrap(),
    }
}
