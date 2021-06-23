// TODO: remove this allow once we have tables
#[allow(unused_imports)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::{collections::HashMap, convert::TryInto};

use crate::database::Sqlite;
use anyhow::{Context, Result};
use covenants::{Lender0, Lender1, LoanRequest, LoanResponse};
use database::LiquidationForm;
use elements::{
    bitcoin::{
        secp256k1::{All, Secp256k1},
        Amount,
    },
    secp256k1_zkp::{
        rand::{CryptoRng, RngCore},
        SecretKey, SECP256K1,
    },
    Address, AssetId, OutPoint, Transaction, Txid,
};
use elements_harness::{elementd_rpc::ElementsRpc, Client as ElementsdClient};
use futures::{stream, stream::FuturesUnordered, Stream, TryStreamExt};
use input::Input;
use serde::{Deserialize, Serialize};
use tokio::sync::watch::Receiver;

mod amounts;

pub mod cli;
pub mod database;
pub mod fixed_rate;
pub mod http;
pub mod kraken;
pub mod models;
pub mod problem;
pub mod schema;

pub use amounts::*;

pub const USDT_ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";

pub struct Bobtimus<R, RS> {
    pub rng: R,
    pub rate_service: RS,
    pub secp: Secp256k1<All>,
    pub elementsd: ElementsdClient,
    pub btc_asset_id: AssetId,
    pub usdt_asset_id: AssetId,
    pub db: Sqlite,
    pub lender_states: HashMap<Txid, Lender1>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSwapPayload {
    pub alice_inputs: Vec<AliceInput>,
    pub address: Address,
    pub amount: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct AliceInput {
    pub outpoint: OutPoint,
    pub blinding_key: SecretKey,
}

impl<R, RS> Bobtimus<R, RS>
where
    R: RngCore + CryptoRng,
    RS: LatestRate,
{
    /// Handle Alice's request to create a swap transaction in which
    /// she buys L-BTC from us and in return we get L-USDt from her.
    pub async fn handle_create_buy_swap(
        &mut self,
        payload: CreateSwapPayload,
    ) -> Result<Transaction> {
        let usdt_amount = LiquidUsdt::from_satodollar(payload.amount);
        let latest_rate = self.rate_service.latest_rate();
        let btc_amount = latest_rate.sell_base(usdt_amount)?;

        let transaction = self
            .swap_transaction(
                (self.usdt_asset_id, usdt_amount.into()),
                (self.btc_asset_id, btc_amount.into()),
                payload.alice_inputs,
                payload.address,
                self.btc_asset_id,
            )
            .await?;

        Ok(transaction)
    }

    /// Handle Alice's request to create a swap transaction in which
    /// she sells L-BTC and we give her L-USDt.
    pub async fn handle_create_sell_swap(
        &mut self,
        payload: CreateSwapPayload,
    ) -> Result<Transaction> {
        let btc_amount = Amount::from_sat(payload.amount);
        let latest_rate = self.rate_service.latest_rate();
        let usdt_amount = latest_rate.buy_quote(btc_amount.into())?;

        let transaction = self
            .swap_transaction(
                (self.btc_asset_id, btc_amount),
                (self.usdt_asset_id, usdt_amount.into()),
                payload.alice_inputs,
                payload.address,
                self.btc_asset_id,
            )
            .await?;

        Ok(transaction)
    }

    async fn find_inputs(
        elements_client: &ElementsdClient,
        asset_id: AssetId,
        input_amount: Amount,
    ) -> Result<Vec<Input>> {
        let bob_inputs = elements_client
            .select_inputs_for(asset_id, input_amount, false)
            .await
            .context("failed to select inputs for swap")?;

        let master_blinding_key = elements_client
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
                mac.update(txout.script_pubkey.as_bytes());

                let result = mac.finalize();
                let input_blinding_sk = SecretKey::from_slice(&result.into_bytes())?;

                Result::<_, anyhow::Error>::Ok(Input {
                    txin: outpoint,
                    original_txout: txout,
                    blinding_key: input_blinding_sk,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(bob_inputs)
    }

    async fn swap_transaction(
        &mut self,
        (alice_input_asset_id, alice_input_amount): (AssetId, Amount),
        (bob_input_asset_id, bob_input_amount): (AssetId, Amount),
        alice_inputs: Vec<AliceInput>,
        alice_address: Address,
        btc_asset_id: AssetId,
    ) -> Result<Transaction> {
        let bob_inputs = Self::find_inputs(&self.elementsd, bob_input_asset_id, bob_input_amount)
            .await
            .context("could not find transaction inputs for Bob")?;

        let bob_address = self
            .elementsd
            .get_new_address(None)
            .await
            .context("failed to get redeem address")?;

        let alice_inputs = alice_inputs
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

                        Result::<_, anyhow::Error>::Ok(Input {
                            txin: outpoint,
                            original_txout: txout,
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
            alice_address,
            bob_input_asset_id,
            bob_input_amount,
        )?;

        let bob = swap::Actor::new(
            &self.secp,
            bob_inputs,
            bob_address,
            alice_input_asset_id,
            alice_input_amount,
        )?;

        let transaction = swap::bob_create_transaction(
            &mut self.rng,
            &self.secp,
            alice,
            bob,
            btc_asset_id,
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

    /// Handle Alice's loan request in which she puts up L-BTC as
    /// collateral and we give lend her L-USDt which she will have to
    /// repay in the future.
    pub async fn handle_loan_request(&mut self, payload: LoanRequest) -> Result<LoanResponse> {
        let lender_address = self
            .elementsd
            .get_new_address(None)
            .await
            .context("failed to get lender address")?;

        let lender0 = Lender0::new(
            &mut self.rng,
            self.btc_asset_id,
            self.usdt_asset_id,
            lender_address,
        )
        .unwrap();

        let lender1 = lender0
            .interpret(
                &mut self.rng,
                &SECP256K1,
                {
                    let elementsd_client = self.elementsd.clone();
                    |amount, asset| async move {
                        Self::find_inputs(&elementsd_client, asset, amount).await
                    }
                },
                payload,
                self.rate_service.latest_rate().bid.as_satodollar(),
            )
            .await
            .unwrap();

        let loan_response = lender1.loan_response();

        self.lender_states
            .insert(loan_response.transaction.txid(), lender1);

        Ok(loan_response)
    }

    /// Handle Alice's request to finalize a loan.
    ///
    /// If we still agree with the loan transaction sent by Alice, we
    /// will sign and broadcast it.
    ///
    /// Additionally, we save the signed liquidation transaction so
    /// that we can broadcast it when the locktime is reached.
    pub async fn finalize_loan(&mut self, transaction: Transaction) -> Result<Txid> {
        // TODO: We should only take into account loan transactions which
        // are relatively recent e.g. within 1 minute. We expect the
        // borrower to quickly perform the protocol and let us broadcast
        // the loan transaction

        let lender = self
            .lender_states
            .get(&transaction.txid())
            .context("unknown loan transaction")?;

        let transaction = lender
            .finalise_loan(transaction, {
                let elementsd = self.elementsd.clone();
                |transaction| async move { elementsd.sign_raw_transaction(&transaction).await }
            })
            .await?;

        let txid = self.elementsd.send_raw_transaction(&transaction).await?;

        let liquidation_tx =
            lender.liquidation_transaction(&mut self.rng, &self.secp, Amount::ONE_SAT)?;
        let locktime = lender
            .timelock
            .try_into()
            .expect("TODO: locktimes should be modelled as u32");

        self.db
            .do_in_transaction(|conn| {
                LiquidationForm::new(txid, &liquidation_tx, locktime).insert(conn)?;

                Ok(())
            })
            .await?;

        Ok(txid)
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
    use elements::{
        bitcoin::{secp256k1::Secp256k1, Amount, Network, PrivateKey, PublicKey},
        secp256k1_zkp::{rand::thread_rng, SecretKey, SECP256K1},
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
    async fn test_handle_btc_sell_swap_request() {
        let db = Sqlite::new_ephemeral_db().expect("A ephemeral db");

        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into()).unwrap(),
                blockchain,
            )
        };
        let mining_address = client.get_new_address(None).await.unwrap();

        let have_asset_id_alice = client.get_bitcoin_asset_id().await.unwrap();
        let have_asset_id_bob = client.issueasset(100_000.0, 0.0, true).await.unwrap().asset;

        let rate_service = fixed_rate::Service::new();
        let redeem_amount_bob = Amount::ONE_BTC;

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
                redeem_amount_bob + Amount::ONE_BTC,
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
        let address = client.get_new_address(None).await.unwrap();
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
            db,
            lender_states: HashMap::new(),
        };

        let transaction = bob
            .handle_create_sell_swap(CreateSwapPayload {
                alice_inputs: vec![AliceInput {
                    outpoint: input_alice.0,
                    blinding_key: fund_blinding_sk_alice,
                }],
                address: final_address_alice,
                amount: redeem_amount_bob.as_sat(),
            })
            .await
            .unwrap();

        let transaction = swap::alice_finalize_transaction(transaction, {
            let value = input_alice.1.value;
            move |mut tx| async move {
                let input_index = tx
                    .input
                    .iter()
                    .position(|txin| fund_alice_txid == txin.previous_output.txid)
                    .context("transaction does not contain input")?;
                let mut cache = SigHashCache::new(&tx);

                tx.input[input_index].witness.script_witness =
                    sign_with_key(&SECP256K1, &mut cache, input_index, &fund_sk_alice, value);

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

        let fee = transaction.fee_in(have_asset_id_bob);
        assert!(utxos.iter().any(|utxo| (utxo.amount
            - (redeem_amount_bob.as_btc() - Amount::from_sat(fee).as_btc()))
        .abs()
            < f64::EPSILON
            && utxo.spendable));
    }

    #[tokio::test]
    async fn test_handle_btc_buy_swap_request() {
        let db = Sqlite::new_ephemeral_db().expect("A ephemeral db");

        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into()).unwrap(),
                blockchain,
            )
        };
        let mining_address = client.get_new_address(None).await.unwrap();

        let have_asset_id_alice = client.issueasset(100_000.0, 0.0, true).await.unwrap().asset;
        let have_asset_id_bob = client.get_bitcoin_asset_id().await.unwrap();

        let rate_service = fixed_rate::Service::new();
        let redeem_amount_bob = LiquidUsdt::from_str_in_dollar("20000.0").unwrap();

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
                Amount::from_btc(10_000.0).unwrap() + redeem_amount_bob.into(),
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

        let mut bob = Bobtimus {
            rng: &mut thread_rng(),
            rate_service,
            secp: Secp256k1::new(),
            elementsd: client.clone(),
            btc_asset_id: have_asset_id_bob,
            usdt_asset_id: have_asset_id_alice,
            db,
            lender_states: HashMap::new(),
        };

        let transaction = bob
            .handle_create_buy_swap(CreateSwapPayload {
                alice_inputs: vec![AliceInput {
                    outpoint: input_alice.0,
                    blinding_key: fund_blinding_sk_alice,
                }],
                address: final_address_alice,
                amount: redeem_amount_bob.as_satodollar(),
            })
            .await
            .unwrap();

        let transaction = swap::alice_finalize_transaction(transaction, {
            let value = input_alice.1.value;
            move |mut tx| async move {
                let input_index = tx
                    .input
                    .iter()
                    .position(|txin| fund_alice_txid == txin.previous_output.txid)
                    .context("transaction does not contain input")?;
                let mut cache = SigHashCache::new(&tx);

                tx.input[input_index].witness.script_witness =
                    sign_with_key(&SECP256K1, &mut cache, input_index, &fund_sk_alice, value);

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

        assert!(utxos.iter().any(
            |utxo| (utxo.amount - Amount::from(redeem_amount_bob).as_btc()).abs() < f64::EPSILON
                && utxo.spendable
        ));
    }

    fn extract_input(tx: &Transaction, address: Address) -> Result<(OutPoint, TxOut)> {
        let vout = tx
            .output
            .iter()
            .position(|output| output.script_pubkey == address.script_pubkey())
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
