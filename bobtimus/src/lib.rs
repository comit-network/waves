use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use elements_fun::{
    bitcoin::Amount,
    secp256k1::{
        rand::{CryptoRng, RngCore},
        SecretKey,
    },
    Address, AssetId, OutPoint, Transaction, TxIn,
};
use elements_harness::{elementd_rpc::ElementsRpc, Client as ElementsdClient};
use futures::{stream::FuturesUnordered, TryStreamExt};
use serde::Deserialize;
use swap::states::{Bob0, Message0};

mod amounts;

pub use amounts::*;

pub static USDT_ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";

#[derive(Clone)]
pub struct Bobtimus<R, RS> {
    pub rng: R,
    pub rate_service: RS,
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
    pub async fn handle_create_swap(
        &mut self,
        payload: CreateSwapPayload,
    ) -> anyhow::Result<Transaction>
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
            .get(0)
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

        let protocol_state = protocol_state.interpret(&mut self.rng, message0)?;
        let tx = self
            .elementsd
            .sign_raw_transaction(protocol_state.unsigned_transaction())
            .await?;

        Ok(tx)
    }
}

#[async_trait]
pub trait LatestRate {
    async fn latest_rate(&self) -> Result<Rate>;
}
