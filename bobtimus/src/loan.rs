use crate::{LiquidUsdt, Rate};
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
    /// Loan term in days
    pub term: u32,
    /// Collateralization in percent
    ///
    /// Rational: If a borrower over-collateralizes with e.g. 150% -> better rate than at 140%
    pub collateralization: Decimal,
    /// Interest rate in percent
    pub interest_rate: Decimal,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoanRequest {
    #[serde(with = "::elements::bitcoin::util::amount::serde::as_sat")]
    pub collateral_amount: Amount,
    collateral_inputs: Vec<Input>,
    #[serde(with = "::elements::bitcoin::util::amount::serde::as_sat")]
    fee_sats_per_vbyte: Amount,
    borrower_pk: PublicKey,
    /// Loan term in days
    pub term: u32,
    borrower_address: Address,
}

impl From<LoanRequest> for baru::loan::LoanRequest {
    fn from(loan_request: LoanRequest) -> Self {
        baru::loan::LoanRequest::new(
            loan_request.collateral_amount,
            loan_request.collateral_inputs,
            loan_request.fee_sats_per_vbyte,
            loan_request.borrower_pk,
            loan_request.borrower_address,
        )
    }
}
