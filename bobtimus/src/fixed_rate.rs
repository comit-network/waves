use crate::{LatestRate, LiquidUsdt, Rate};
use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use std::{convert::TryFrom, time::Duration};
use tokio::{
    sync::watch::{self, Receiver},
    time::delay_for,
};

#[derive(Clone)]
pub struct Service(Receiver<Rate>);

impl Service {
    pub fn new() -> Self {
        let data = fixed_rate();
        let (tx, rx) = watch::channel(data);

        tokio::spawn(async move {
            loop {
                let _ = tx.broadcast(data);

                delay_for(Duration::from_secs(5)).await;
            }
        });

        Self(rx)
    }

    pub fn subscribe(&self) -> impl Stream<Item = Rate> + Clone {
        self.0.clone()
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LatestRate for Service {
    async fn latest_rate(&mut self) -> Result<Rate> {
        Ok(fixed_rate())
    }
}

fn fixed_rate() -> Rate {
    Rate {
        ask: LiquidUsdt::try_from(20_000.0).unwrap(),
        bid: LiquidUsdt::try_from(19_000.0).unwrap(),
    }
}
