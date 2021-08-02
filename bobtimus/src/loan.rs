use crate::{LiquidUsdt, Rate};
use elements::bitcoin::Amount;
use rust_decimal::Decimal;

#[derive(Debug, Clone, serde::Serialize)]
pub struct LoanOffer {
    // TODO: Potentially add an id if we want to track offers - for now we just check if an incoming request is acceptable
    pub rate: Rate,

    #[serde(with = "::elements::bitcoin::util::amount::serde::as_sat")]
    pub fee_sats_per_vbyte: Amount,

    #[serde(serialize_with = "LiquidUsdt::serialize_to_nominal")]
    pub min_principal: LiquidUsdt,
    #[serde(serialize_with = "LiquidUsdt::serialize_to_nominal")]
    pub max_principal: LiquidUsdt,

    /// The maximum LTV that defines at what point the lender liquidates
    ///
    /// LTV ... loan to value
    /// LTV = principal_amount/loan_value
    /// where:
    ///     principal_amount: the amount lent out
    ///     loan_value: the amount of collateral
    ///
    /// Simple Example (interest / fees not taken into account):
    ///
    /// The borrower takes out a loan at:
    ///     max_ltv = 0.7 (70%)
    ///     rate: 1 BTC = $100
    ///     principal_amount: $100
    ///     collateral: 2 BTC = $200 (over-collateralized by 200%)
    ///     current LTV = 100 / 200 = 0.5 (50%)
    /// Since the actual LTV 0.5 < 0.7, so all is good.
    ///
    /// Let's say Bitcoin value falls to $70:
    ///     LTV = 100 / 2 * 70 => 100 / 140 = 0.71
    /// The actual LTV 0.71 > 0.7 so the lender liquidates.
    ///
    /// The max_ltv protects the lender from Bitcoin falling too much.
    pub max_ltv: Decimal,

    pub interest: Vec<Interest>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Interest {
    /// Timelock in blocks
    pub timelock: u32,
    /// Interest rate in percent
    pub interest_rate: Decimal,
}
