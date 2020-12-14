use anyhow::{anyhow, Result};
use elements_fun::bitcoin::Amount;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt::Debug};

/// Prices at which 1 L-BTC will be traded, in L-USDt.
///
/// - The `ask` represents the minimum price for which we are willing to sell 1 L-BTC.
/// - The `bid` represents the maximum price we are willing pay for 1 L-BTC.
#[derive(Default, Debug, Clone, Copy, Serialize, PartialEq)]
pub struct Rate {
    pub ask: LiquidUsdt,
    pub bid: LiquidUsdt,
}

impl Rate {
    pub fn buy_quote(&self, base: LiquidBtc) -> Result<LiquidUsdt> {
        Self::quote(self.bid, base)
    }

    pub fn sell_quote(&self, base: LiquidBtc) -> Result<LiquidUsdt> {
        Self::quote(self.ask, base)
    }

    fn quote(rate: LiquidUsdt, base: LiquidBtc) -> Result<LiquidUsdt> {
        let sats = base.0.as_sat();
        let btc = Decimal::from(sats)
            .checked_div(Decimal::from(Amount::ONE_BTC.as_sat()))
            .ok_or_else(|| anyhow!("division overflow"))?;

        let satodollars_per_btc = Decimal::from(rate.as_satodollar());
        let satodollars = satodollars_per_btc * btc;
        let satodollars = satodollars
            .to_u64()
            .ok_or_else(|| anyhow!("decimal cannot be represented as u64"))?;

        Ok(LiquidUsdt::from_satodollar(satodollars))
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Default)]
pub struct LiquidUsdt(
    #[serde(with = "::elements_fun::bitcoin::util::amount::serde::as_sat")] Amount,
);

impl LiquidUsdt {
    /// Create an amount with satodollar precision and the given number of satodollars.
    ///
    /// The originally named "satodollar" is the smallest division of
    /// L-USDt that can be represented in Liquid.
    pub fn from_satodollar(satodollars: u64) -> Self {
        Self(Amount::from_sat(satodollars))
    }

    /// Get the number of hundred-millionths of L-USDt in this amount.
    ///
    /// The originally named "satodollar" is the smallest division of
    /// L-USDt that can be represented in Liquid.
    pub fn as_satodollar(&self) -> u64 {
        self.0.as_sat()
    }

    pub fn from_str_in_dollar(s: &str) -> Result<Self> {
        let amount = Amount::from_str_in(s, elements_fun::bitcoin::Denomination::Bitcoin)?;

        Ok(Self(amount))
    }
}

impl Debug for LiquidUsdt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LiquidUsdt({} dollars)", self.0.as_btc())
    }
}

impl From<LiquidUsdt> for Amount {
    fn from(from: LiquidUsdt) -> Self {
        from.0
    }
}

impl TryFrom<f64> for LiquidUsdt {
    type Error = anyhow::Error;

    fn try_from(value: f64) -> Result<Self> {
        Ok(LiquidUsdt(Amount::from_btc(value)?))
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct LiquidBtc(
    #[serde(with = "::elements_fun::bitcoin::util::amount::serde::as_sat")] Amount,
);

impl From<Amount> for LiquidBtc {
    fn from(amount: Amount) -> Self {
        Self(amount)
    }
}

impl From<LiquidBtc> for Amount {
    fn from(amount: LiquidBtc) -> Self {
        amount.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sell_quote() {
        let rate = Rate {
            ask: LiquidUsdt::try_from(19_313.52).unwrap(),
            bid: LiquidUsdt::try_from(19_213.52).unwrap(),
        };

        let btc_amount = LiquidBtc(Amount::from_btc(2.5).unwrap());

        let usdt_amount = rate.sell_quote(btc_amount).unwrap();

        assert_eq!(usdt_amount, LiquidUsdt::try_from(48_283.80).unwrap())
    }

    #[test]
    fn buy_quote() {
        let rate = Rate {
            ask: LiquidUsdt::try_from(19_313.52).unwrap(),
            bid: LiquidUsdt::try_from(19_213.52).unwrap(),
        };

        let btc_amount = LiquidBtc(Amount::from_btc(2.5).unwrap());

        let usdt_amount = rate.buy_quote(btc_amount).unwrap();

        assert_eq!(usdt_amount, LiquidUsdt::try_from(48_033.80).unwrap())
    }
}
