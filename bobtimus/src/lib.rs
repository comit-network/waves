use anyhow::{Context, Result};
use elements_fun::{
    bitcoin::{
        secp256k1::{All, Secp256k1},
        Amount,
    },
    secp256k1::{
        rand::{CryptoRng, RngCore},
        SecretKey,
    },
    Address, AssetId, OutPoint, Transaction, TxIn,
};
use elements_harness::{elementd_rpc::ElementsRpc, Client as ElementsdClient};
use futures::{stream, stream::FuturesUnordered, Stream, TryStreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::watch::Receiver;

mod amounts;

pub mod cli;
pub mod fixed_rate;
pub mod http;
pub mod kraken;
pub mod problem;

pub use amounts::*;

pub const USDT_ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";

pub struct Bobtimus<R, RS> {
    pub rng: R,
    pub rate_service: RS,
    pub secp: Secp256k1<All>,
    pub elementsd: ElementsdClient,
    pub btc_asset_id: AssetId,
    pub usdt_asset_id: AssetId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSwapPayload {
    pub alice_inputs: Vec<AliceInput>,
    pub address: Address,
    pub btc_amount: LiquidBtc,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct AliceInput {
    pub outpoint: OutPoint,
    pub blinding_key: SecretKey,
}

impl<R, RS> Bobtimus<R, RS> {
    pub async fn handle_create_swap(&mut self, payload: CreateSwapPayload) -> Result<Transaction>
    where
        R: RngCore + CryptoRng,
        RS: LatestRate,
    {
        let latest_rate = self.rate_service.latest_rate();
        let usdt_amount = latest_rate.buy_quote(payload.btc_amount)?;

        let bob_inputs = self
            .elementsd
            .select_inputs_for(self.usdt_asset_id, usdt_amount.into(), false)
            .await
            .context("failed to select inputs for swap")?;

        let master_blinding_key = self
            .elementsd
            .dumpmasterblindingkey()
            .await
            .context("failed to dump master blinding key")?;
        let master_blinding_key = hex::decode(master_blinding_key)?;

        let bob_inputs = bob_inputs
            .into_iter()
            .map(|(outpoint, txout)| {
                use hmac::{Hmac, Mac, NewMac};
                use sha2::Sha256;

                let mut mac = Hmac::<Sha256>::new_varkey(&master_blinding_key)
                    .expect("HMAC can take key of any size");
                mac.update(txout.script_pubkey().as_bytes());

                let result = mac.finalize();
                let input_blinding_sk = SecretKey::from_slice(&result.into_bytes())?;

                Result::<_, anyhow::Error>::Ok(swap::Input {
                    txin: TxIn {
                        previous_output: outpoint,
                        is_pegin: false,
                        has_issuance: false,
                        script_sig: Default::default(),
                        sequence: 0,
                        asset_issuance: Default::default(),
                        witness: Default::default(),
                    },
                    txout,
                    blinding_key: input_blinding_sk,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let bob_address = self
            .elementsd
            .getnewaddress()
            .await
            .context("failed to get redeem address")?;

        let alice_inputs = payload
            .alice_inputs
            .iter()
            .copied()
            .map(
                |AliceInput {
                     outpoint,
                     blinding_key,
                 }| {
                    let client = self.elementsd.clone();
                    async move {
                        let transaction = client
                            .get_raw_transaction(outpoint.txid)
                            .await
                            .with_context(|| {
                                format!("failed to fetch transaction {}", outpoint.txid)
                            })?;

                        let txin = TxIn {
                            previous_output: outpoint,
                            is_pegin: false,
                            has_issuance: false,
                            script_sig: Default::default(),
                            sequence: 0,
                            asset_issuance: Default::default(),
                            witness: Default::default(),
                        };
                        let txout = transaction
                            .output
                            .get(outpoint.vout as usize)
                            .with_context(|| {
                                format!(
                                    "vout index {} is not valid for transaction {}",
                                    outpoint.vout, outpoint.txid
                                )
                            })?
                            .clone();

                        Result::<_, anyhow::Error>::Ok(swap::Input {
                            txin,
                            txout,
                            blinding_key,
                        })
                    }
                },
            )
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        let alice = swap::Actor::new(
            &self.secp,
            alice_inputs,
            payload.address,
            self.usdt_asset_id,
            usdt_amount.into(),
        )?;
        let bob = swap::Actor::new(
            &self.secp,
            bob_inputs,
            bob_address,
            self.btc_asset_id,
            payload.btc_amount.into(),
        )?;

        let transaction = swap::bob_create_transaction(
            &mut self.rng,
            &self.secp,
            alice,
            bob,
            self.btc_asset_id,
            Amount::from_sat(1), // TODO: Make this dynamic once there is something going on on Liquid
            {
                let elementsd = self.elementsd.clone();
                move |transaction| async move {
                    let tx = elementsd.sign_raw_transaction(&transaction).await?;

                    Result::<_, anyhow::Error>::Ok(tx)
                }
            },
        )
        .await?;

        Ok(transaction)
    }
}

pub trait LatestRate {
    fn latest_rate(&mut self) -> Rate;
}

#[derive(Clone)]
pub struct RateSubscription {
    receiver: Receiver<Rate>,
}

impl From<Receiver<Rate>> for RateSubscription {
    fn from(receiver: Receiver<Rate>) -> Self {
        Self { receiver }
    }
}

impl RateSubscription {
    pub fn into_stream(self) -> impl Stream<Item = Result<Rate>> {
        stream::try_unfold(self.receiver, |mut receiver| async move {
            receiver
                .changed()
                .await
                .context("failed to receive latest rate update")?;

            let latest_rate = *receiver.borrow();

            Ok(Some((latest_rate, receiver)))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_rate;
    use anyhow::{Context, Result};
    use elements_fun::{
        bitcoin::{secp256k1::Secp256k1, Amount, Network, PrivateKey, PublicKey},
        secp256k1::{rand::thread_rng, SecretKey, SECP256K1},
        sighash::SigHashCache,
        Address, AddressParams, OutPoint, Transaction, TxOut,
    };
    use elements_harness::{
        elementd_rpc::{ElementsRpc, ListUnspentOptions},
        Client, Elementsd,
    };
    use swap::sign_with_key;
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
        let have_asset_id_bob = client.issueasset(100_000.0, 0.0, true).await.unwrap().asset;

        let rate_service = fixed_rate::Service::new();
        let redeem_amount_bob = LiquidBtc::from(Amount::ONE_BTC);

        let (
            fund_address_alice,
            fund_sk_alice,
            _fund_pk_alice,
            fund_blinding_sk_alice,
            _fund_blinding_pk_alice,
        ) = make_confidential_address();

        let fund_alice_txid = client
            .send_asset_to_address(
                &fund_address_alice,
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
            _final_blinding_sk_alice,
            _final_blinding_pk_alice,
        ) = make_confidential_address();

        // move issued asset to wallet address
        let address = client.getnewaddress().await.unwrap();
        let _txid = client
            .send_asset_to_address(
                &address,
                Amount::from_btc(10.0).unwrap(),
                Some(have_asset_id_bob),
            )
            .await
            .unwrap();
        client.generatetoaddress(1, &mining_address).await.unwrap();

        let mut bob = Bobtimus {
            rng: &mut thread_rng(),
            rate_service,
            secp: Secp256k1::new(),
            elementsd: client.clone(),
            btc_asset_id: have_asset_id_alice,
            usdt_asset_id: have_asset_id_bob,
        };

        let transaction = bob
            .handle_create_swap(CreateSwapPayload {
                alice_inputs: vec![AliceInput {
                    outpoint: input_alice.0,
                    blinding_key: fund_blinding_sk_alice,
                }],
                address: final_address_alice,
                btc_amount: redeem_amount_bob,
            })
            .await
            .unwrap();

        let transaction = swap::alice_finalize_transaction(transaction, {
            let commitment = input_alice.1.into_confidential().unwrap().value;
            move |mut tx| async move {
                let input_index = tx
                    .input
                    .iter()
                    .position(|txin| fund_alice_txid == txin.previous_output.txid)
                    .context("transaction does not contain input")?;
                let mut cache = SigHashCache::new(&tx);

                tx.input[input_index].witness.script_witness = sign_with_key(
                    &SECP256K1,
                    &mut cache,
                    input_index,
                    &fund_sk_alice,
                    commitment,
                );

                Ok(tx)
            }
        })
        .await
        .unwrap();

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
        let txout = tx.output[vout].clone();
        Ok((outpoint, txout))
    }

    fn make_keypair() -> (SecretKey, PublicKey) {
        let sk = SecretKey::new(&mut thread_rng());
        let pk = PublicKey::from_private_key(
            &SECP256K1,
            &PrivateKey {
                compressed: true,
                network: Network::Regtest,
                key: sk,
            },
        );

        (sk, pk)
    }

    fn make_confidential_address() -> (Address, SecretKey, PublicKey, SecretKey, PublicKey) {
        let (sk, pk) = make_keypair();
        let (blinding_sk, blinding_pk) = make_keypair();

        (
            Address::p2wpkh(&pk, Some(blinding_pk.key), &AddressParams::ELEMENTS),
            sk,
            pk,
            blinding_sk,
            blinding_pk,
        )
    }
}
