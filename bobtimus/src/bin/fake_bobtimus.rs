use anyhow::Result;
use bobtimus::{Bobtimus, CreateSwapPayload, USDT_ASSET_ID};
use elements_fun::{
    secp256k1::rand::{rngs::StdRng, thread_rng, SeedableRng},
    AssetId,
};
use elements_harness::Client;
use futures::StreamExt;
use reqwest::Url;
use std::convert::Infallible;
use structopt::StructOpt;
use warp::{Filter, Rejection, Reply};

#[derive(structopt::StructOpt, Debug)]
#[structopt(name = "fake-bobtimus", about = "Definitely not Bobtimus")]
pub struct StartCommand {
    #[structopt(default_value = "http://127.0.0.1:7042", long = "elementsd")]
    elementsd_url: Url,
    #[structopt(default_value = "3030")]
    api_port: u16,
    #[structopt(
        default_value = USDT_ASSET_ID,
        long = "usdt"
    )]
    usdt_asset_id: AssetId,
}

#[tokio::main]
async fn main() -> Result<()> {
    let StartCommand {
        elementsd_url,
        api_port,
        usdt_asset_id,
    } = StartCommand::from_args();

    let elementsd = Client::new(elementsd_url.into_string())?;
    let btc_asset_id = elementsd.get_bitcoin_asset_id().await?;

    let rate_service = fixed_rate::Service::new();

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
    let latest_rate = warp::path("rate/lbtc-lusdt").and(warp::get()).map(move || {
        warp::sse::reply(
            warp::sse::keep_alive().stream(subscription.clone().map(|data| {
                Result::<_, Infallible>::Ok((warp::sse::event("rate"), warp::sse::json(data)))
            })),
        )
    });

    let create_swap = warp::post()
        .and(warp::path("swap/lbtc-lusdt"))
        .and(warp::path::end())
        .and(bobtimus_filter.clone())
        .and(warp::body::json())
        .and_then(create_swap);

    warp::serve(latest_rate.or(create_swap))
        .run(([127, 0, 0, 1], api_port))
        .await;

    Ok(())
}

pub async fn create_swap(
    mut bobtimus: Bobtimus<StdRng, fixed_rate::Service>,
    payload: CreateSwapPayload,
) -> Result<impl Reply, Rejection> {
    bobtimus
        .handle_create_swap(payload)
        .await
        .map(|transaction| warp::reply::json(&transaction))
        .map_err(|_| warp::reject::reject())
}

mod fixed_rate {
    use anyhow::Result;
    use async_trait::async_trait;
    use bobtimus::{LatestRate, LiquidUsdt, Rate};
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
        async fn latest_rate(&self) -> Result<Rate> {
            Ok(fixed_rate())
        }
    }

    fn fixed_rate() -> Rate {
        Rate {
            ask: LiquidUsdt::try_from(20_000.0).unwrap(),
            bid: LiquidUsdt::try_from(19_000.0).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use bobtimus::{Bobtimus, CreateSwapPayload, LatestRate, LiquidBtc};
    use elements_fun::{
        bitcoin::Amount, secp256k1::rand::thread_rng, Address, OutPoint, Transaction, TxOut,
    };
    use elements_harness::{
        elementd_rpc::{ElementsRpc, ListUnspentOptions},
        Client, Elementsd,
    };
    use swap::{
        make_confidential_address,
        states::{Alice0, Message1},
    };
    use testcontainers::clients::Cli;

    #[tokio::test]
    async fn test_handle_swap_request() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };
        let mining_address = client.getnewaddress().await.unwrap();

        let have_asset_id_alice = client.get_bitcoin_asset_id().await.unwrap();
        let have_asset_id_bob = client.issueasset(10.0, 0.0, true).await.unwrap().asset;

        let rate_service = fixed_rate::Service::new();
        let redeem_amount_bob = LiquidBtc::from(Amount::ONE_BTC);

        let rate = rate_service.latest_rate().await.unwrap();
        let redeem_amount_alice = rate.buy_quote(redeem_amount_bob).unwrap();

        let (
            fund_address_alice,
            fund_sk_alice,
            _fund_pk_alice,
            fund_blinding_sk_alice,
            _fund_blinding_pk_alice,
        ) = make_confidential_address();

        let fund_alice_txid = client
            .send_asset_to_address(
                fund_address_alice.clone(),
                Amount::from(redeem_amount_bob) + Amount::ONE_BTC,
                Some(have_asset_id_alice),
            )
            .await
            .unwrap();
        client.generatetoaddress(1, &mining_address).await.unwrap();

        let input_alice = extract_input(
            &client.get_raw_transaction(fund_alice_txid).await.unwrap(),
            fund_address_alice,
        )
        .unwrap();

        let (
            final_address_alice,
            _final_sk_alice,
            _final_pk_alice,
            final_blinding_sk_alice,
            _final_blinding_pk_alice,
        ) = make_confidential_address();

        let (
            change_address_alice,
            _change_sk_alice,
            _change_pk_alice,
            change_blinding_sk_alice,
            _change_blinding_pk_alice,
        ) = make_confidential_address();

        // move issued asset to wallet address
        let address = client.getnewaddress().await.unwrap();
        let _txid = client
            .send_asset_to_address(
                address,
                Amount::from_btc(10.0).unwrap(),
                Some(have_asset_id_bob),
            )
            .await
            .unwrap();
        client.generatetoaddress(1, &mining_address).await.unwrap();

        let fee = Amount::from_sat(10_000);

        let alice = Alice0::new(
            redeem_amount_alice.into(),
            redeem_amount_bob.into(),
            input_alice,
            fund_sk_alice,
            fund_blinding_sk_alice,
            have_asset_id_bob,
            final_address_alice.clone(),
            final_blinding_sk_alice,
            change_address_alice.clone(),
            change_blinding_sk_alice,
            fee,
        );

        let message0 = alice.compose();

        let mut bob = Bobtimus {
            rng: &mut thread_rng(),
            rate_service,
            elementsd: client.clone(),
            btc_asset_id: have_asset_id_alice,
            usdt_asset_id: have_asset_id_bob,
        };

        let transaction = bob
            .handle_create_swap(CreateSwapPayload {
                protocol_msg: message0,
                btc_amount: redeem_amount_bob,
            })
            .await
            .unwrap();

        let transaction = alice.interpret(Message1 { transaction }).unwrap();

        let _txid = client.send_raw_transaction(&transaction).await.unwrap();
        let _txid = client.generatetoaddress(1, &mining_address).await.unwrap();

        let utxos = client
            .listunspent(
                None,
                None,
                None,
                None,
                Some(ListUnspentOptions {
                    asset: Some(have_asset_id_alice),
                    ..Default::default()
                }),
            )
            .await
            .unwrap();

        let error = 0.0001;
        assert!(utxos.iter().any(
            |utxo| (utxo.amount - Amount::from(redeem_amount_bob).as_btc()).abs() < error
                && utxo.spendable
        ));
    }

    fn extract_input(tx: &Transaction, address: Address) -> Result<(OutPoint, TxOut)> {
        let vout = tx
            .output
            .iter()
            .position(|output| output.script_pubkey() == &address.script_pubkey())
            .context("Tx doesn't pay to address")?;

        let outpoint = OutPoint {
            txid: tx.txid(),
            vout: vout as u32,
        };
        let tx_out = tx.output[vout].clone();
        Ok((outpoint, tx_out))
    }
}
