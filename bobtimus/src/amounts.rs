use anyhow::{anyhow, Context, Result};
use elements::bitcoin::{Amount, Denomination};
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal, RoundingStrategy,
};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, fmt::Debug};

/// Prices at which 1 L-BTC will be traded, in L-USDt.
///
/// - The `ask` represents the minimum price for which we are willing to sell 1 L-BTC.
/// - The `bid` represents the maximum price we are willing pay for 1 L-BTC.
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct Rate {
    #[serde(serialize_with = "LiquidUsdt::serialize_to_nominal")]
    pub ask: LiquidUsdt,
    #[serde(serialize_with = "LiquidUsdt::serialize_to_nominal")]
    pub bid: LiquidUsdt,
}

impl Rate {
    pub const ZERO: Rate = Rate {
        ask: LiquidUsdt(Amount::ZERO),
        bid: LiquidUsdt(Amount::ZERO),
    };

    pub fn buy_quote(&self, base: LiquidBtc) -> Result<LiquidUsdt> {
        let sats = base.0.as_sat();
        let btc = Decimal::from(sats)
            .checked_div(Decimal::from(Amount::ONE_BTC.as_sat()))
            .ok_or_else(|| anyhow!("division overflow"))?;

        let satodollars_per_btc = Decimal::from(self.bid.as_satodollar());
        let satodollars = satodollars_per_btc * btc;
        let satodollars = satodollars
            .to_u64()
            .ok_or_else(|| anyhow!("decimal cannot be represented as u64"))?;

        Ok(LiquidUsdt::from_satodollar(satodollars))
    }

    pub fn sell_base(&self, quote: LiquidUsdt) -> Result<LiquidBtc> {
        let satodollars = quote.as_satodollar();

        let btc = Decimal::from(satodollars)
            .checked_div(Decimal::from(self.ask.as_satodollar()))
            .ok_or_else(|| anyhow!("division overflow"))?;

        // we round to 1 satoshi
        let btc = btc.round_dp_with_strategy(8, RoundingStrategy::MidpointAwayFromZero);

        let btc = btc
            .to_f64()
            .ok_or_else(|| anyhow!("decimal cannot be represented as f64"))?;
        let btc = LiquidBtc::from(
            Amount::from_btc(btc)
                .with_context(|| format!("bitcoin amount cannot be parsed from float {}", btc))?,
        );

        Ok(btc)
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Serialize, Deserialize, Default)]
pub struct LiquidUsdt(#[serde(with = "::elements::bitcoin::util::amount::serde::as_sat")] Amount);

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
        let amount = Amount::from_str_in(s, elements::bitcoin::Denomination::Bitcoin)?;

        Ok(Self(amount))
    }

    pub fn serialize_to_nominal<S>(amount: &LiquidUsdt, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let float = &amount.0.to_float_in(Denomination::Bitcoin);
        let rounded = format!("{:.2}", float);
        let rounded: f64 = rounded.parse().expect("valid float");

        serializer.serialize_f64(rounded)
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
        let value = Decimal::from_f64(value)
            .with_context(|| format!("LiquidUsdt amount cannot be parsed from float {}", value))?
            .round_dp_with_strategy(8, RoundingStrategy::MidpointAwayFromZero)
            .to_f64()
            .unwrap();
        Ok(LiquidUsdt(Amount::from_btc(value)?))
    }
}

impl fmt::Display for LiquidUsdt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_value_in(f, Denomination::Bitcoin)?;
        write!(f, " L-USDT")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LiquidBtc(
    #[serde(with = "::elements::bitcoin::util::amount::serde::as_sat")] pub Amount,
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
    fn buy_quote() {
        let rate = Rate {
            ask: LiquidUsdt::try_from(19_313.52).unwrap(),
            bid: LiquidUsdt::try_from(19_213.52).unwrap(),
        };

        let btc_amount = LiquidBtc(Amount::from_btc(2.5).unwrap());

        let usdt_amount = rate.buy_quote(btc_amount).unwrap();

        assert_eq!(usdt_amount, LiquidUsdt::try_from(48_033.80).unwrap())
    }

    #[test]
    fn sell_base() {
        let rate = Rate {
            ask: LiquidUsdt::try_from(19_313.52).unwrap(),
            bid: LiquidUsdt::try_from(19_213.52).unwrap(),
        };

        let usdt_amount = LiquidUsdt::from_str_in_dollar("9656.76").unwrap();
        let btc_amount = rate.sell_base(usdt_amount).unwrap();

        assert_eq!(btc_amount, LiquidBtc(Amount::from_btc(0.5).unwrap()))
    }

    #[test]
    fn rate_serialized_with_nominal_unit() {
        let rate = Rate {
            ask: LiquidUsdt::try_from(19_313.524).unwrap(),
            bid: LiquidUsdt::try_from(19_213.525).unwrap(),
        };
        let serialized = serde_json::to_string(&rate).unwrap();

        assert_eq!(serialized, "{\"ask\":19313.52,\"bid\":19213.53}")
    }

    #[test]
    fn test_rounding_liquid_usdt() {
        let amount = LiquidUsdt::try_from(0.0000000123).unwrap();
        assert_eq!(amount.0, Amount::from_sat(1));
    }
}
