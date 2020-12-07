use anyhow::{bail, Result};
use async_trait::async_trait;
use elements_fun::{
    bitcoin::Amount,
    secp256k1::rand::CryptoRng,
    secp256k1::{rand::RngCore, SecretKey},
    AssetId, Transaction,
};
use elements_harness::{elementd_rpc::ElementsRpc, Client as ElementsdClient};
use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde::Deserialize;
use std::{convert::TryFrom, ops::Mul};
use swap::states::{Bob0, Message0};

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
    pub btc_amount: BitcoinAmount,
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
        let usdt_amount = latest_rate * payload.btc_amount;

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
            payload.btc_amount.0,
            usdt_amount.0,
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
    async fn latest_rate(&self) -> Result<Rate>;
}

/// Rate in USDt per bitcoin.
#[derive(Debug, Clone, Copy)]
pub struct Rate(pub Decimal);

#[derive(Clone, Copy, PartialEq)]
pub struct LiquidUsdt(Amount);

impl TryFrom<f64> for LiquidUsdt {
    type Error = anyhow::Error;

    fn try_from(value: f64) -> Result<Self> {
        Ok(LiquidUsdt(Amount::from_btc(value)?))
    }
}

impl From<LiquidUsdt> for Amount {
    fn from(from: LiquidUsdt) -> Self {
        from.0
    }
}

impl std::fmt::Debug for LiquidUsdt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LiquidUsdt({} dollars)", self.0.as_btc())
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct BitcoinAmount(
    #[serde(with = "::elements_fun::bitcoin::util::amount::serde::as_sat")] Amount,
);

impl Mul<BitcoinAmount> for Rate {
    type Output = LiquidUsdt;

    fn mul(self, rhs: BitcoinAmount) -> Self::Output {
        let amount = self.0.mul(Decimal::from(rhs.0.as_sat())).to_u64().unwrap();

        LiquidUsdt(Amount::from_sat(amount))
    }
}

impl From<Amount> for BitcoinAmount {
    fn from(amount: Amount) -> Self {
        Self(amount)
    }
}

impl From<BitcoinAmount> for Amount {
    fn from(amount: BitcoinAmount) -> Self {
        amount.0
    }
}

// TODO: This controls how we serialise the Rate. It would be better
// if we could use serde instead
impl std::fmt::Display for Rate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_and_btc_amount_multiplication() {
        // 19_313.52 USDt per bitcoin
        let rate = Rate(Decimal::new(1_931_352, 2));

        let btc_amount = BitcoinAmount(Amount::from_btc(2.5).unwrap());

        let usdt_amount = rate * btc_amount;

        assert_eq!(usdt_amount, LiquidUsdt::try_from(48_283.80).unwrap())
    }
}
