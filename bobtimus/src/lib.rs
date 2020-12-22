use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use elements_fun::{
    bitcoin::Amount,
    secp256k1::{
        rand::{CryptoRng, RngCore},
        SecretKey,
    },
    Address, AssetId, OutPoint, TxIn,
};
use elements_harness::{elementd_rpc::ElementsRpc, Client as ElementsdClient};
use futures::{stream::FuturesUnordered, TryStreamExt};
use serde::Deserialize;
use swap::{Bob0, Message0, Message1};

mod amounts;

pub mod cli;
pub mod fixed_rate;
pub mod http;
pub mod kraken;

pub use amounts::*;
use elements_fun::bitcoin::secp256k1::{All, Secp256k1};

pub const USDT_ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";

#[derive(Clone)]
pub struct Bobtimus<R, RS> {
    pub rng: R,
    pub rate_service: RS,
    pub secp: Secp256k1<All>,
    pub elementsd: ElementsdClient,
    pub btc_asset_id: AssetId,
    pub usdt_asset_id: AssetId,
}

#[derive(Deserialize)]
pub struct CreateSwapPayload {
    pub alice_inputs: Vec<AliceInput>,
    pub address_redeem: Address,
    pub address_change: Address,
    #[serde(with = "::elements_fun::bitcoin::util::amount::serde::as_sat")]
    pub fee: Amount,
    pub btc_amount: LiquidBtc,
}

#[derive(Deserialize, Clone, Copy)]
pub struct AliceInput {
    pub outpoint: OutPoint,
    pub blinding_key: SecretKey,
}

impl<R, RS> Bobtimus<R, RS> {
    pub async fn handle_create_swap(&mut self, payload: CreateSwapPayload) -> Result<Message1>
    where
        R: RngCore + CryptoRng,
        RS: LatestRate,
    {
        let latest_rate = self
            .rate_service
            .latest_rate()
            .await
            .context("failed to get latest rate")?;
        let usdt_amount = latest_rate.buy_quote(payload.btc_amount)?;

        let bob_inputs = self
            .elementsd
            .select_inputs_for(self.usdt_asset_id, usdt_amount.into(), true)
            .await
            .context("failed to select inputs for swap")?;

        let (input, input_blinding_sk) = match bob_inputs.as_slice() {
            [(outpoint, txout)] => {
                use hmac::{Hmac, Mac, NewMac};
                use sha2::Sha256;

                let master_blinding_key = self
                    .elementsd
                    .dumpmasterblindingkey()
                    .await
                    .context("failed to dump master blinding key")?;
                let master_blinding_key = hex::decode(master_blinding_key)?;

                let mut mac = Hmac::<Sha256>::new_varkey(&master_blinding_key)
                    .expect("HMAC can take key of any size");
                mac.update(txout.script_pubkey().as_bytes());

                let result = mac.finalize();
                let input_blinding_sk = SecretKey::from_slice(&result.into_bytes())?;

                ((*outpoint, txout.clone()), input_blinding_sk)
            }
            [] => bail!("found no inputs"),
            _ => bail!("TODO: Support multiple inputs per party"),
        };

        let redeem_address = self
            .elementsd
            .getnewaddress()
            .await
            .context("failed to get redeem address")?;
        let change_address = self
            .elementsd
            .getnewaddress()
            .await
            .context("failed to get change address")?;

        let protocol_state = Bob0::new(
            usdt_amount.into(),
            payload.btc_amount.into(),
            input,
            input_blinding_sk,
            self.btc_asset_id,
            redeem_address,
            change_address,
        );

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
                        let txin_as_txout = transaction
                            .output
                            .get(outpoint.vout as usize)
                            .with_context(|| {
                                format!(
                                    "vout index {} is not valid for transaction {}",
                                    outpoint.vout, outpoint.txid
                                )
                            })?
                            .clone();

                        Result::<_, anyhow::Error>::Ok((txin, txin_as_txout, blinding_key))
                    }
                },
            )
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        let (input, input_as_txout, input_blinding_sk) = alice_inputs
            .get(0) // TODO: Handle multiple inputs from Alice
            .context("alice needs to send at least one input")?
            .clone();

        let message0 = Message0 {
            input,
            input_as_txout,
            input_blinding_sk,
            address_redeem: payload.address_redeem,
            address_change: payload.address_change,
            fee: payload.fee,
        };

        let bob1 = protocol_state.interpret(&mut self.rng, &self.secp, message0)?;

        let elementsd = self.elementsd.clone();
        let signer = bob1.sign_with_wallet(move |transaction| async move {
            let tx = elementsd.sign_raw_transaction(transaction).await?;

            Result::<_, anyhow::Error>::Ok(tx)
        });

        let message = bob1.compose(signer).await?;

        Ok(message)
    }
}

#[async_trait]
pub trait LatestRate {
    async fn latest_rate(&mut self) -> Result<Rate>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_rate;
    use anyhow::{Context, Result};
    use elements_fun::{
        bitcoin::{secp256k1::Secp256k1, Amount, Network, PrivateKey, PublicKey},
        secp256k1::{rand::thread_rng, SecretKey, SECP256K1},
        Address, AddressParams, OutPoint, Transaction, TxOut,
    };
    use elements_harness::{
        elementd_rpc::{ElementsRpc, ListUnspentOptions},
        Client, Elementsd,
    };
    use swap::Alice0;
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

        let mut rate_service = fixed_rate::Service::new();
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
                &address,
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
            secp: Secp256k1::new(),
            elementsd: client.clone(),
            btc_asset_id: have_asset_id_alice,
            usdt_asset_id: have_asset_id_bob,
        };

        let message1 = bob
            .handle_create_swap(CreateSwapPayload {
                alice_inputs: vec![AliceInput {
                    outpoint: message0.input.previous_output,
                    blinding_key: message0.input_blinding_sk,
                }],
                address_redeem: message0.address_redeem,
                address_change: message0.address_change,
                fee: message0.fee,
                btc_amount: redeem_amount_bob,
            })
            .await
            .unwrap();

        let transaction = alice.interpret(message1).unwrap();

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
