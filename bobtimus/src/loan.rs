use crate::{LiquidBtc, LiquidUsdt, Rate};
use baru::input::Input;
use elements::{
    bitcoin::{Amount, PublicKey},
    Address,
};
use rust_decimal::Decimal;

#[derive(Debug, Clone, serde::Serialize)]
pub struct LoanOffer {
    // TODO: Potentially add an id if we want to track offers - for now we just check if an incoming request is acceptable
    pub rate: Rate,

    #[serde(with = "::elements::bitcoin::util::amount::serde::as_sat")]
    pub fee_sats_per_vbyte: Amount,

    pub min_principal: LiquidUsdt, // USDT
    pub max_principal: LiquidUsdt, // USDT

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

    /// Base interest in percent (to be applied to the principal amount)
    pub base_interest_rate: Decimal,

    /// Interest in relation to terms
    pub terms: Vec<Term>,

    /// Interest rates in relation to collteralization
    pub collateralizations: Vec<Collateralization>,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Term {
    pub days: u32,
    /// Interest to be added on top of the base interest rate for this term
    pub interest_mod: Decimal,
}

/// Allows to specify a better rate for users that
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Collateralization {
    pub collateralization: Decimal,
    /// Interest to be added on top of the base interest rate for this term.
    pub interest_mod: Decimal,
}

// TODO: Make sure that removing sat_per_vbyte is OK here
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoanRequest {
    pub principal_amount: LiquidUsdt, // USDT
    pub collateral_amount: LiquidBtc, // BTC
    pub collateral_inputs: Vec<Input>,
    pub collateralization: Decimal,
    pub borrower_pk: PublicKey,
    /// Loan term in days
    pub term: u32,
    pub borrower_address: Address,
}
