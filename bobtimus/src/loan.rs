use crate::{LiquidBtc, LiquidUsdt, Rate};
use anyhow::{Context, Result};
use baru::input::Input;
use elements::{
    bitcoin::{Amount, PublicKey},
    Address,
};
use rust_decimal::{prelude::ToPrimitive, Decimal};

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
    pub principal_amount: LiquidUsdt,
    pub collateral_amount: LiquidBtc,
    pub collateral_inputs: Vec<Input>,
    pub collateralization: Decimal,
    pub borrower_pk: PublicKey,
    /// Loan term in days
    pub term: u32,
    pub borrower_address: Address,
}

pub fn calculate_interest_rate(
    borrower_term: u32,
    borrower_collateralization: Decimal,
    term_thresholds: &[Term],
    collateralization_thresholds: &[Collateralization],
    base_interest_rate: Decimal,
) -> Result<Decimal> {
    let mut term_interest_mod = Decimal::ZERO;
    for term in term_thresholds {
        if borrower_term >= term.days {
            term_interest_mod = term.interest_mod;
            continue;
        }
        break;
    }

    let mut collateralization_interest_mod = Decimal::ZERO;
    for collateralization in collateralization_thresholds {
        if borrower_collateralization >= collateralization.collateralization {
            collateralization_interest_mod = collateralization.interest_mod;
            continue;
        }
        break;
    }

    let interest_rate = base_interest_rate
        .checked_add(term_interest_mod)
        .context("Overflow due to addition")?
        .checked_add(collateralization_interest_mod)
        .context("Overflow due to addition")?;

    Ok(interest_rate)
}

pub fn calculate_repayment_amount(
    principal_amount: LiquidUsdt,
    interest_percentage: Decimal,
) -> Result<LiquidUsdt> {
    let principal_amount = Decimal::from(principal_amount.as_satodollar());

    let repayment_amount = principal_amount
        .checked_add(
            principal_amount
                .checked_mul(interest_percentage)
                .context("multiplication overflow")?,
        )
        .context("addition overflow")?;
    let repayment_amount = LiquidUsdt::from_satodollar(
        repayment_amount
            .to_u64()
            .context("decimal cannot be represented as u64")?,
    );

    Ok(repayment_amount)
}

pub fn calculate_liquidation_price(
    repayment_amount: LiquidUsdt,
    collateral_amount: LiquidBtc,
) -> Result<LiquidUsdt> {
    let repayment_amount = Decimal::from(repayment_amount.as_satodollar());
    let one_btc_as_sat = Decimal::from(Amount::ONE_BTC.as_sat());
    let collateral_as_btc = Decimal::from(collateral_amount.0.as_sat())
        .checked_div(one_btc_as_sat)
        .context("division error")?;

    let liquidation_price = repayment_amount
        .checked_div(collateral_as_btc)
        .context("division error")?;
    let liquidation_price = LiquidUsdt::from_satodollar(
        liquidation_price
            .to_u64()
            .context("decimal cannot be represented as u64")?,
    );

    Ok(liquidation_price)
}

pub fn calculate_request_price(
    repayment_amount: LiquidUsdt,
    collateral_amount: LiquidBtc,
    collateralization: Decimal,
) -> Result<LiquidUsdt> {
    let repayment_amount = Decimal::from(repayment_amount.as_satodollar());

    let one_btc_as_sat = Decimal::from(Amount::ONE_BTC.as_sat());
    let collateral_as_btc = Decimal::from(collateral_amount.0.as_sat())
        .checked_div(one_btc_as_sat)
        .context("division error")?;

    let price = repayment_amount
        .checked_div(
            collateral_as_btc
                .checked_div(collateralization)
                .context("division error")?,
        )
        .context("division error")?;
    let price = LiquidUsdt::from_satodollar(
        price
            .to_u64()
            .context("decimal cannot be represented as u64")?,
    );

    Ok(price)
}

pub fn calculate_ltv(
    repayment_amount: LiquidUsdt,
    collateral_amount: LiquidBtc,
    current_bid_price: LiquidUsdt,
) -> Result<Decimal> {
    let repayment_amount = Decimal::from(repayment_amount.as_satodollar());
    let price = Decimal::from(current_bid_price.as_satodollar());

    let one_btc = Decimal::from(Amount::ONE_BTC.as_sat());
    let collateral_in_btc = Decimal::from(collateral_amount.0.as_sat())
        .checked_div(one_btc)
        .context("division error")?;

    let ltv = repayment_amount
        .checked_div(
            collateral_in_btc
                .checked_mul(price)
                .context("multiplication error")?,
        )
        .context("division error")?;

    Ok(ltv)
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum LoanValidationError {
    #[error(
        "The given price {request_price} is not acceptable with current price {current_price}"
    )]
    PriceNotAcceptable {
        request_price: LiquidUsdt,
        current_price: LiquidUsdt,
    },

    #[error("The given principal amount {request_principal} is below the configured minimum {min_principal}")]
    PrincipalBelowMin {
        request_principal: LiquidUsdt,
        min_principal: LiquidUsdt,
    },

    #[error("The given principal amount {request_principal} is above the configured maximum {max_principal}")]
    PrincipalAboveMax {
        request_principal: LiquidUsdt,
        max_principal: LiquidUsdt,
    },

    #[error("The LTV value {request_ltv} is above the configured maximum {max_ltv}")]
    LtvAboveMax {
        request_ltv: Decimal,
        max_ltv: Decimal,
    },
}

#[allow(clippy::too_many_arguments)]
pub fn validate_loan_is_acceptable(
    request_price: LiquidUsdt,
    current_price: LiquidUsdt,
    price_fluctuation_interval: (Decimal, Decimal),
    request_principal: LiquidUsdt,
    min_principal: LiquidUsdt,
    max_principal: LiquidUsdt,
    request_ltv: Decimal,
    max_ltv: Decimal,
) -> Result<Result<(), LoanValidationError>> {
    let request_price_dec = Decimal::from(request_price.as_satodollar());
    let current_price_dec = Decimal::from(current_price.as_satodollar());

    // TODO: Evaluate if we want to use an upper and a lower bound.
    //  We could just restrict by upper bound, because that is what makes it more expensive for the lender
    //  i.e. if price was 1000 and is 100 now we must ensure to accept only if the current price it not higher than 100 + x%
    let (lower, upper) = price_fluctuation_interval;
    let lower_bound = current_price_dec
        .checked_mul(lower)
        .context("multiplication error")?;
    let upper_bound = current_price_dec
        .checked_mul(upper)
        .context("multiplication error")?;

    if request_price_dec < lower_bound || request_price_dec > upper_bound {
        return Ok(Err(LoanValidationError::PriceNotAcceptable {
            request_price,
            current_price,
        }));
    }

    if request_principal < min_principal {
        return Ok(Err(LoanValidationError::PrincipalBelowMin {
            request_principal,
            min_principal,
        }));
    }

    if request_principal > max_principal {
        return Ok(Err(LoanValidationError::PrincipalAboveMax {
            request_principal,
            max_principal,
        }));
    }

    if request_ltv > max_ltv {
        return Ok(Err(LoanValidationError::LtvAboveMax {
            request_ltv,
            max_ltv,
        }));
    }

    Ok(Ok(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::proptest;
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_interest_rate() {
        let term_thresholds = vec![Term {
            days: 30,
            interest_mod: dec!(0.001),
        }];
        let collateralization_thresholds = vec![Collateralization {
            collateralization: dec!(1.5),
            interest_mod: dec!(-0.002),
        }];
        let base_interest_rate = dec!(0.05);

        let borrower_term = 30;
        let borrower_collateralization = dec!(1.5);
        let interest_rate = calculate_interest_rate(
            borrower_term,
            borrower_collateralization,
            &term_thresholds,
            &collateralization_thresholds,
            base_interest_rate,
        )
        .unwrap();
        assert_eq!(interest_rate, dec!(0.049));

        let borrower_term = 29;
        let borrower_collateralization = dec!(1.4);
        let interest_rate = calculate_interest_rate(
            borrower_term,
            borrower_collateralization,
            &term_thresholds,
            &collateralization_thresholds,
            base_interest_rate,
        )
        .unwrap();
        assert_eq!(interest_rate, dec!(0.05));

        let borrower_term = 30;
        let borrower_collateralization = dec!(1.4);
        let interest_rate = calculate_interest_rate(
            borrower_term,
            borrower_collateralization,
            &term_thresholds,
            &collateralization_thresholds,
            base_interest_rate,
        )
        .unwrap();
        assert_eq!(interest_rate, dec!(0.051));

        let borrower_term = 29;
        let borrower_collateralization = dec!(1.5);
        let interest_rate = calculate_interest_rate(
            borrower_term,
            borrower_collateralization,
            &term_thresholds,
            &collateralization_thresholds,
            base_interest_rate,
        )
        .unwrap();
        assert_eq!(interest_rate, dec!(0.048));
    }

    #[test]
    fn test_calculate_repayment_amount() {
        let principal_amount = LiquidUsdt::from_satodollar(10000);
        let interest_percentage = dec!(0.05);

        let repayment_amount =
            calculate_repayment_amount(principal_amount, interest_percentage).unwrap();
        assert_eq!(repayment_amount, LiquidUsdt::from_satodollar(10500));
    }

    proptest! {
        #[test]
        fn test_calculate_repayment_amount_no_panic(
            // we eventually hit decimal limits, but the amounts are so high that is should not matter
            principal_amount in 1u64..15_000_000_000_000_000_000, // satdollar ^= 150 billion usd
            interest_percentage in 0.001f32..0.2,
        ) {
            let principal_amount = LiquidUsdt::from_satodollar(principal_amount);
            let interest_percentage = Decimal::from_f32(interest_percentage).unwrap();

            let _ = calculate_repayment_amount(principal_amount, interest_percentage).unwrap();
        }
    }

    #[test]
    fn test_calculate_liquidation_price() {
        let repayment_amount = LiquidUsdt::from_str_in_dollar("10500").unwrap();
        let collateral = LiquidBtc::from(Amount::from_btc(0.35).unwrap());

        let liquidation_price = calculate_liquidation_price(repayment_amount, collateral).unwrap();

        assert_eq!(
            liquidation_price,
            LiquidUsdt::from_str_in_dollar("30000").unwrap()
        )
    }

    proptest! {
        #[test]
        fn test_calculate_liquidation_price_no_panic(
            repayment_amount in 1u64..,
            collateral in 1u64..,
        ) {
            let repayment_amount = LiquidUsdt::from_satodollar(repayment_amount);
            let collateral = LiquidBtc::from(Amount::from_sat(collateral));

            let _ = calculate_liquidation_price(repayment_amount, collateral).unwrap();
        }
    }

    #[test]
    fn test_calculate_price() {
        let repayment_amount = LiquidUsdt::from_str_in_dollar("10500").unwrap();
        let collateral = LiquidBtc::from(Amount::from_btc(0.39375).unwrap());
        let collateralization = dec!(1.5);

        let price =
            calculate_request_price(repayment_amount, collateral, collateralization).unwrap();

        assert_eq!(price, LiquidUsdt::from_str_in_dollar("40000").unwrap());
    }

    #[test]
    fn test_calculate_ltv() {
        let repayment_amount = LiquidUsdt::from_str_in_dollar("10500").unwrap();
        let collateral = LiquidBtc::from(Amount::from_btc(0.4).unwrap());
        let current_price = LiquidUsdt::from_str_in_dollar("40000").unwrap();
        let ltv = calculate_ltv(repayment_amount, collateral, current_price).unwrap();

        assert_eq!(ltv, dec!(0.65625))
    }

    #[test]
    fn given_loan_request_acceptable_then_dont_error() {
        let request_price = LiquidUsdt::from_str_in_dollar("40000").unwrap();
        let current_price = LiquidUsdt::from_str_in_dollar("39603.96039604").unwrap();
        let price_fluctuation_interval = (dec!(0.90), dec!(1.01));
        let request_principal = LiquidUsdt::from_str_in_dollar("1000").unwrap();
        let min_principal = LiquidUsdt::from_str_in_dollar("1000").unwrap();
        let max_principal = LiquidUsdt::from_str_in_dollar("10000").unwrap();
        let request_ltv = dec!(0.8);
        let max_ltv = dec!(0.8);

        validate_loan_is_acceptable(
            request_price,
            current_price,
            price_fluctuation_interval,
            request_principal,
            min_principal,
            max_principal,
            request_ltv,
            max_ltv,
        )
        .unwrap()
        .unwrap();
    }

    #[test]
    fn given_loan_request_and_price_drop_lower_then_fluctuation_then_error() {
        let request_price = LiquidUsdt::from_str_in_dollar("40000").unwrap();
        let price_fluctuation_interval = (dec!(0.90), dec!(1.01));
        let request_principal = LiquidUsdt::from_str_in_dollar("1000").unwrap();
        let min_principal = LiquidUsdt::from_str_in_dollar("1000").unwrap();
        let max_principal = LiquidUsdt::from_str_in_dollar("10000").unwrap();
        let request_ltv = dec!(0.8);
        let max_ltv = dec!(0.8);

        let current_price = LiquidUsdt::from_str_in_dollar("39603.96039603").unwrap();
        let error = validate_loan_is_acceptable(
            request_price,
            current_price,
            price_fluctuation_interval,
            request_principal,
            min_principal,
            max_principal,
            request_ltv,
            max_ltv,
        )
        .unwrap()
        .unwrap_err();

        assert_eq!(
            error,
            LoanValidationError::PriceNotAcceptable {
                request_price,
                current_price
            }
        )
    }

    #[test]
    fn given_loan_request_and_price_raise_higher_then_fluctuation_then_error() {
        let request_price = LiquidUsdt::from_str_in_dollar("40000").unwrap();
        let price_fluctuation_interval = (dec!(0.90), dec!(1.01));
        let request_principal = LiquidUsdt::from_str_in_dollar("1000").unwrap();
        let min_principal = LiquidUsdt::from_str_in_dollar("1000").unwrap();
        let max_principal = LiquidUsdt::from_str_in_dollar("10000").unwrap();
        let request_ltv = dec!(0.8);
        let max_ltv = dec!(0.8);

        let current_price = LiquidUsdt::from_str_in_dollar("44444.44444445").unwrap();
        let error = validate_loan_is_acceptable(
            request_price,
            current_price,
            price_fluctuation_interval,
            request_principal,
            min_principal,
            max_principal,
            request_ltv,
            max_ltv,
        )
        .unwrap()
        .unwrap_err();

        assert_eq!(
            error,
            LoanValidationError::PriceNotAcceptable {
                request_price,
                current_price
            }
        )
    }

    #[test]
    fn given_loan_request_with_principal_lower_min_then_error() {
        let request_price = LiquidUsdt::from_str_in_dollar("40000").unwrap();
        let current_price = LiquidUsdt::from_str_in_dollar("40000").unwrap();
        let price_fluctuation_interval = (dec!(0.90), dec!(1.01));
        let min_principal = LiquidUsdt::from_str_in_dollar("1000").unwrap();
        let max_principal = LiquidUsdt::from_str_in_dollar("10000").unwrap();
        let request_ltv = dec!(0.8);
        let max_ltv = dec!(0.8);

        let request_principal = LiquidUsdt::from_str_in_dollar("999.99999999").unwrap();
        let error = validate_loan_is_acceptable(
            request_price,
            current_price,
            price_fluctuation_interval,
            request_principal,
            min_principal,
            max_principal,
            request_ltv,
            max_ltv,
        )
        .unwrap()
        .unwrap_err();

        assert_eq!(
            error,
            LoanValidationError::PrincipalBelowMin {
                request_principal,
                min_principal
            }
        )
    }
}
