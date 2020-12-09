use anyhow::Result;
use elements_fun::bitcoin::Amount;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt::Debug, ops::Mul};

/// Prices at which 1 L-BTC will be traded, in L-USDt.
///
/// - The `ask` represents the minimum price for which we are willing to sell 1 L-BTC.
/// - The `bid` represents the maximum price we are willing pay for 1 L-BTC.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Rate {
    pub ask: Decimal,
    pub bid: Decimal,
}

impl Rate {
    pub fn buy_quote(&self, base: LiquidBtc) -> LiquidUsdt {
        Self::quote(self.bid, base)
    }

    pub fn sell_quote(&self, base: LiquidBtc) -> LiquidUsdt {
        Self::quote(self.ask, base)
    }

    fn quote(rate: Decimal, base: LiquidBtc) -> LiquidUsdt {
        let amount = rate.mul(Decimal::from(base.0.as_sat())).to_u64().unwrap();

        LiquidUsdt(Amount::from_sat(amount))
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct LiquidUsdt(Amount);

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
            ask: Decimal::new(1_931_352, 2),
            bid: Decimal::new(1_921_352, 2),
        };

        let btc_amount = LiquidBtc(Amount::from_btc(2.5).unwrap());

        let usdt_amount = rate.sell_quote(btc_amount);

        assert_eq!(usdt_amount, LiquidUsdt::try_from(48_283.80).unwrap())
    }

    #[test]
    fn buy_quote() {
        let rate = Rate {
            ask: Decimal::new(1_931_352, 2),
            bid: Decimal::new(1_921_352, 2),
        };

        let btc_amount = LiquidBtc(Amount::from_btc(2.5).unwrap());

        let usdt_amount = rate.buy_quote(btc_amount);

        assert_eq!(usdt_amount, LiquidUsdt::try_from(48_033.80).unwrap())
    }
}
