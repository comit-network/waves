use anyhow::{bail, Result};
use async_trait::async_trait;
use elements_fun::{
    secp256k1::{
        rand::{CryptoRng, RngCore},
        SecretKey,
    },
    AssetId, Transaction,
};
use elements_harness::{elementd_rpc::ElementsRpc, Client as ElementsdClient};
use serde::Deserialize;
use swap::states::{Bob0, Message0};

mod amounts;

pub mod cli;
pub mod http;

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
    #[serde(flatten)]
    pub protocol_msg: Message0,
    pub btc_amount: LiquidBtc,
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
        let latest_rate = self.rate_service.latest_rate().await?;
        let usdt_amount = latest_rate.buy_quote(payload.btc_amount)?;

        let inputs = self
            .elementsd
            .select_inputs_for(self.usdt_asset_id, usdt_amount.into(), true)
            .await?;

        let (input, input_blinding_sk) = match inputs.as_slice() {
            [(outpoint, txout)] => {
                use hmac::{Hmac, Mac, NewMac};
                use sha2::Sha256;

                let master_blinding_key = self.elementsd.dumpmasterblindingkey().await?;
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

        let redeem_address = self.elementsd.getnewaddress().await?;
        let change_address = self.elementsd.getnewaddress().await?;

        let protocol_state = Bob0::new(
            usdt_amount.into(),
            payload.btc_amount.into(),
            input,
            input_blinding_sk,
            self.btc_asset_id,
            redeem_address,
            change_address,
        );

        let protocol_state = protocol_state.interpret(&mut self.rng, payload.protocol_msg)?;
        let tx = self
            .elementsd
            .sign_raw_transaction(protocol_state.unsigned_transaction())
            .await?;

        Ok(tx)
    }
}

#[async_trait]
pub trait LatestRate {
    async fn latest_rate(&mut self) -> Result<Rate>;
}
