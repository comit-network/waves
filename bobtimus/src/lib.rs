#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::collections::HashMap;

use crate::{
    database::{queries, Sqlite},
    elements_rpc::{Client, ElementsRpc},
};
use anyhow::{Context, Result};
use baru::{
    input::Input,
    loan::{Lender0, Lender1, LoanResponse},
    swap,
};
use database::LiquidationForm;
use elements::{
    bitcoin::{
        secp256k1::{All, Secp256k1},
        Amount,
    },
    secp256k1_zkp::{
        rand::{CryptoRng, RngCore},
        SecretKey, SECP256K1,
    },
    Address, AssetId, OutPoint, Transaction, Txid,
};
use futures::{stream, stream::FuturesUnordered, Stream, TryStreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::watch::Receiver;

mod amounts;

pub mod cli;
pub mod database;
pub mod elements_rpc;
pub mod fixed_rate;
pub mod http;
pub mod kraken;
pub mod loan;
pub mod problem;
pub mod schema;

use crate::loan::{Collateralization, LoanOffer, LoanRequest, Term};
pub use amounts::*;
use elements::bitcoin::PublicKey;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use rust_decimal_macros::dec;
use std::{
    convert::TryFrom,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const USDT_ASSET_ID: &str = "ce091c998b83c78bb71a632313ba3760f1763d9cfcffae02258ffa9865a37bd2";

pub struct Bobtimus<R, RS> {
    pub rng: R,
    pub rate_service: RS,
    pub secp: Secp256k1<All>,
    pub elementsd: Client,
    pub btc_asset_id: AssetId,
    pub usdt_asset_id: AssetId,
    pub db: Sqlite,
    pub lender_states: HashMap<Txid, Lender1>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSwapPayload {
    pub alice_inputs: Vec<AliceInput>,
    pub address: Address,
    pub amount: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct AliceInput {
    pub outpoint: OutPoint,
    pub blinding_key: SecretKey,
}

impl<R, RS> Bobtimus<R, RS>
where
    R: RngCore + CryptoRng,
    RS: LatestRate,
{
    /// Handle Alice's request to create a swap transaction in which
    /// she buys L-BTC from us and in return we get L-USDt from her.
    pub async fn handle_create_buy_swap(
        &mut self,
        payload: CreateSwapPayload,
    ) -> Result<Transaction> {
        let usdt_amount = LiquidUsdt::from_satodollar(payload.amount);
        let latest_rate = self.rate_service.latest_rate();
        let btc_amount = latest_rate.sell_base(usdt_amount)?;

        let transaction = self
            .swap_transaction(
                (self.usdt_asset_id, usdt_amount.into()),
                (self.btc_asset_id, btc_amount.into()),
                payload.alice_inputs,
                payload.address,
                self.btc_asset_id,
            )
            .await?;

        Ok(transaction)
    }

    /// Handle Alice's request to create a swap transaction in which
    /// she sells L-BTC and we give her L-USDt.
    pub async fn handle_create_sell_swap(
        &mut self,
        payload: CreateSwapPayload,
    ) -> Result<Transaction> {
        let btc_amount = Amount::from_sat(payload.amount);
        let latest_rate = self.rate_service.latest_rate();
        let usdt_amount = latest_rate.buy_quote(btc_amount.into())?;

        let transaction = self
            .swap_transaction(
                (self.btc_asset_id, btc_amount),
                (self.usdt_asset_id, usdt_amount.into()),
                payload.alice_inputs,
                payload.address,
                self.btc_asset_id,
            )
            .await?;

        Ok(transaction)
    }

    async fn find_inputs(
        elements_client: &Client,
        asset_id: AssetId,
        input_amount: Amount,
    ) -> Result<Vec<Input>> {
        let bob_inputs = elements_client
            .select_inputs_for(asset_id, input_amount, false)
            .await
            .context("failed to select inputs for swap")?;

        let master_blinding_key = elements_client
            .dumpmasterblindingkey()
            .await
            .context("failed to dump master blinding key")?;

        let master_blinding_key = hex::decode(master_blinding_key)?;

        let bob_inputs = bob_inputs
            .into_iter()
            .map(|(outpoint, txout)| {
                use hmac::{Hmac, Mac, NewMac};
                use sha2::Sha256;

                let mut mac = Hmac::<Sha256>::new_varkey(&master_blinding_key)
                    .expect("HMAC can take key of any size");
                mac.update(txout.script_pubkey.as_bytes());

                let result = mac.finalize();
                let input_blinding_sk = SecretKey::from_slice(&result.into_bytes())?;

                Result::<_, anyhow::Error>::Ok(Input {
                    txin: outpoint,
                    original_txout: txout,
                    blinding_key: input_blinding_sk,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(bob_inputs)
    }

    async fn swap_transaction(
        &mut self,
        (alice_input_asset_id, alice_input_amount): (AssetId, Amount),
        (bob_input_asset_id, bob_input_amount): (AssetId, Amount),
        alice_inputs: Vec<AliceInput>,
        alice_address: Address,
        btc_asset_id: AssetId,
    ) -> Result<Transaction> {
        let bob_inputs = Self::find_inputs(&self.elementsd, bob_input_asset_id, bob_input_amount)
            .await
            .context("could not find transaction inputs for Bob")?;

        let bob_address = self
            .elementsd
            .get_new_segwit_confidential_address()
            .await
            .context("failed to get redeem address")?;

        let alice_inputs = alice_inputs
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

                        let txout = transaction
                            .output
                            .get(outpoint.vout as usize)
                            .with_context(|| {
                                format!(
                                    "vout index {} is not valid for transaction {}",
                                    outpoint.vout, outpoint.txid
                                )
                            })?
                            .clone();

                        Result::<_, anyhow::Error>::Ok(Input {
                            txin: outpoint,
                            original_txout: txout,
                            blinding_key,
                        })
                    }
                },
            )
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        let alice = swap::Actor::new(
            &self.secp,
            alice_inputs,
            alice_address,
            bob_input_asset_id,
            bob_input_amount,
        )?;

        let bob = swap::Actor::new(
            &self.secp,
            bob_inputs,
            bob_address,
            alice_input_asset_id,
            alice_input_amount,
        )?;

        let transaction = swap::bob_create_transaction(
            &mut self.rng,
            &self.secp,
            alice,
            bob,
            btc_asset_id,
            Amount::from_sat(1), // TODO: Make this dynamic once there is something going on on Liquid
            {
                let elementsd = self.elementsd.clone();
                move |transaction| async move {
                    let tx = elementsd.sign_raw_transaction(&transaction).await?;

                    Result::<_, anyhow::Error>::Ok(tx)
                }
            },
        )
        .await?;

        Ok(transaction)
    }

    /// Handle the borrower's loan offer request
    ///
    /// We return the range of possible loan terms to the borrower.
    /// The borrower can then request a loan using parameters that are within our terms.
    pub async fn handle_loan_offer_request(&mut self) -> Result<LoanOffer> {
        Ok(self.current_loan_offer())
    }

    fn current_loan_offer(&mut self) -> LoanOffer {
        LoanOffer {
            rate: self.rate_service.latest_rate(),
            // TODO: Dynamic fee estimation
            fee_sats_per_vbyte: Amount::from_sat(1),
            min_principal: LiquidUsdt::from_str_in_dollar("100")
                .expect("static value to be convertible"),
            max_principal: LiquidUsdt::from_str_in_dollar("10000")
                .expect("static value to be convertible"),
            // TODO: Maximum LTV to be decided by a model
            max_ltv: dec!(0.8),
            // TODO: Interest to be decided by a model
            base_interest_rate: dec!(0.05),
            // TODO: Potentially fine-tune the model with these values
            terms: vec![
                Term {
                    days: 30,
                    interest_mod: Decimal::ZERO,
                },
                Term {
                    days: 60,
                    interest_mod: dec!(0.10),
                },
                Term {
                    days: 120,
                    interest_mod: dec!(0.15),
                },
            ],
            collateralizations: vec![
                Collateralization {
                    collateralization: dec!(1.5),
                    interest_mod: dec!(-0.01),
                },
                Collateralization {
                    collateralization: dec!(2.0),
                    interest_mod: dec!(-0.02),
                },
            ],
        }
    }

    /// Handle the borrower's loan request in which she puts up L-BTC as
    /// collateral and we lend L-USDt to her which she will have to
    /// repay in the future.
    pub async fn handle_loan_request(&mut self, loan_request: LoanRequest) -> Result<LoanResponse> {
        let loan_offer = self.current_loan_offer();

        let interest_rate = calculate_interest_rate(
            loan_request.term,
            loan_request.collateralization,
            &loan_offer.terms,
            &loan_offer.collateralizations,
            loan_offer.base_interest_rate,
        )?;
        let repayment_amount =
            calculate_repayment_amount(loan_request.principal_amount, interest_rate)?;

        let request_price = calculate_request_price(
            repayment_amount,
            loan_request.collateral_amount,
            loan_request.collateralization,
        )?;

        let current_price = self.rate_service.latest_rate();
        let request_ltv = calculate_ltv(
            repayment_amount,
            loan_request.collateral_amount,
            current_price.bid,
        )?;

        // TODO: Make configurable
        let price_fluctuation_interval = (dec!(0.99), dec!(1.01));

        validate_loan_is_acceptable(
            request_price,
            current_price.bid,
            price_fluctuation_interval,
            loan_request.principal_amount,
            loan_offer.min_principal,
            loan_offer.max_principal,
            request_ltv,
            loan_offer.max_ltv,
        )??;

        let oracle_secret_key = elements::secp256k1_zkp::key::ONE_KEY;
        let oralce_priv_key = elements::bitcoin::PrivateKey::new(
            oracle_secret_key,
            elements::bitcoin::Network::Regtest,
        );
        let oracle_pk = PublicKey::from_private_key(&self.secp, &oralce_priv_key);

        let timelock = days_to_unix_timestamp_timelock(loan_request.term, SystemTime::now())?;

        let lender_address = self
            .elementsd
            .get_new_segwit_confidential_address()
            .await
            .context("failed to get lender address")?;

        let address_blinder = self
            .elementsd
            .get_address_blinding_key(&lender_address)
            .await?;

        let lender0 = Lender0::new(
            &mut self.rng,
            self.btc_asset_id,
            self.usdt_asset_id,
            lender_address,
            address_blinder,
            oracle_pk,
        )
        .unwrap();

        let elementsd_client = self.elementsd.clone();
        let principal_inputs = Self::find_inputs(
            &elementsd_client,
            self.usdt_asset_id,
            loan_request.principal_amount.into(),
        )
        .await?;

        let liquidation_price =
            calculate_liquidation_price(repayment_amount, loan_request.collateral_amount)?;

        let lender1 = lender0
            .build_loan_transaction(
                &mut self.rng,
                &SECP256K1,
                loan_offer.fee_sats_per_vbyte,
                (
                    loan_request.collateral_amount.into(),
                    loan_request.collateral_inputs,
                ),
                (loan_request.principal_amount.into(), principal_inputs),
                repayment_amount.into(),
                liquidation_price.as_satodollar(),
                (loan_request.borrower_pk, loan_request.borrower_address),
                timelock,
            )
            .await
            .context("Failed to build loan transaction")?;

        let loan_response = lender1.loan_response();

        self.lender_states
            .insert(loan_response.transaction().txid(), lender1);

        Ok(loan_response)
    }

    /// Handle the borrower's request to finalize a loan.
    ///
    /// If we still agree with the loan transaction sent by the borrower, we
    /// will sign and broadcast it.
    ///
    /// Additionally, we save the signed liquidation transaction so
    /// that we can broadcast it when the locktime is reached.
    pub async fn finalize_loan(&mut self, transaction: Transaction) -> Result<Txid> {
        // TODO: We should only take into account loan transactions which
        // are relatively recent e.g. within 1 minute. We expect the
        // borrower to quickly perform the protocol and let us broadcast
        // the loan transaction

        let lender = self
            .lender_states
            .get(&transaction.txid())
            .context("unknown loan transaction")?;

        let transaction = lender
            .finalise_loan(transaction, {
                let elementsd = self.elementsd.clone();
                |transaction| async move { elementsd.sign_raw_transaction(&transaction).await }
            })
            .await?;

        let txid = self.elementsd.send_raw_transaction(&transaction).await?;

        let liquidation_tx = lender
            .liquidation_transaction(&mut self.rng, &self.secp, Amount::ONE_SAT)
            .await?;
        let locktime = lender.collateral_contract().timelock();

        self.db
            .do_in_transaction(|conn| {
                LiquidationForm::new(txid, &liquidation_tx, *locktime).insert(conn)?;

                Ok(())
            })
            .await?;

        Ok(txid)
    }
}

pub trait LatestRate {
    fn latest_rate(&mut self) -> Rate;
}

#[derive(Clone)]
pub struct RateSubscription {
    receiver: Receiver<Rate>,
}

impl From<Receiver<Rate>> for RateSubscription {
    fn from(receiver: Receiver<Rate>) -> Self {
        Self { receiver }
    }
}

impl RateSubscription {
    pub fn into_stream(self) -> impl Stream<Item = Result<Rate>> {
        stream::try_unfold(self.receiver, |mut receiver| async move {
            receiver
                .changed()
                .await
                .context("failed to receive latest rate update")?;

            let latest_rate = *receiver.borrow();

            Ok(Some((latest_rate, receiver)))
        })
    }
}

pub async fn liquidate_loans(elementsd: &Client, db: Sqlite) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let secs_since_epoch = now.as_secs();

    let liquidation_txs = db
        .do_in_transaction(|conn| {
            let txs = queries::get_publishable_liquidations_txs(conn, secs_since_epoch)?;
            Ok(txs)
        })
        .await?;

    for tx in liquidation_txs.iter() {
        match elementsd.send_raw_transaction(&tx).await {
            Ok(txid) => log::info!("Broadcast liquidation transaction {}", txid),
            Err(e) => log::error!("Failed to broadcast liquidation transaction: {}", e),
        };
    }

    Ok(())
}
/// Calculates the absolute timelock from the loan term in days
///
/// The timelock is represented as Unix timestamp (seconds since the epoch).
/// Note: Miniscript uses u32 for representing the timestamp so we return a u32.
fn days_to_unix_timestamp_timelock(term_in_days: u32, now: SystemTime) -> Result<u32> {
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    let term = Duration::from_secs((term_in_days * 24 * 60 * 60) as u64);

    let timelock = (since_the_epoch + term).as_secs();
    let timelock = u32::try_from(timelock)
        .context("Overflow, the given timestamp appears to be too far in the future")?;

    Ok(timelock)
}

fn calculate_interest_rate(
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

fn calculate_repayment_amount(
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

fn calculate_liquidation_price(
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

fn calculate_request_price(
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

fn calculate_ltv(
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
enum LoanValidationError {
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
fn validate_loan_is_acceptable(
    request_price: LiquidUsdt,
    current_price: LiquidUsdt,
    price_fluctuation_interval: (Decimal, Decimal),
    request_principal: LiquidUsdt,
    min_principal: LiquidUsdt,
    max_principal: LiquidUsdt,
    request_ltv: Decimal,
    max_ltv: Decimal,
) -> Result<Result<(), LoanValidationError>> {
    let request_price_dec = request_price.as_satodollar_dec();
    let current_price_dec = current_price.as_satodollar_dec();

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
    use crate::{
        elements_rpc::{Client, ElementsRpc, ListUnspentOptions},
        fixed_rate,
    };
    use anyhow::{Context, Result};
    use baru::swap::sign_with_key;
    use elements::{
        bitcoin::{secp256k1::Secp256k1, Amount, Network, PrivateKey, PublicKey},
        secp256k1_zkp::{rand::thread_rng, SecretKey, SECP256K1},
        sighash::SigHashCache,
        Address, AddressParams, OutPoint, Transaction, TxOut,
    };
    use elements_harness::Elementsd;
    use proptest::proptest;
    use rust_decimal::prelude::FromPrimitive;
    use testcontainers::clients::Cli;

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

    // This test ensures that this function will not panic on different systems now and in the future.
    // At the point of writing 30868 days were supported, equivalent to 84.569863 calendar years.
    // We allow a maximum of 18250 days = 50 years for loan terms.
    // This test will pass for the next ~34.5 years given a correct system time.
    proptest! {
        #[test]
        fn timelock_calculation_does_not_panic_between_1_day_and_100_years(
            term_in_days in 1u32..18250, // 18250 days = 50 years
        ) {
            let now = SystemTime::now();
            let _ = days_to_unix_timestamp_timelock(term_in_days, now).unwrap();
        }
    }

    #[test]
    fn timelock_calculation_30_days() {
        let term_in_days = 30;
        let now = SystemTime::now();

        let since_epoch = u32::try_from(now.duration_since(UNIX_EPOCH).unwrap().as_secs()).unwrap();

        let timelock = days_to_unix_timestamp_timelock(term_in_days, now).unwrap();

        let difference = timelock - since_epoch;

        // 2_592_000 = 30 days in secs
        assert_eq!(difference, 2_592_000)
    }

    #[tokio::test]
    async fn test_handle_btc_sell_swap_request() {
        let db = Sqlite::new_ephemeral_db().expect("A ephemeral db");

        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into()).unwrap(),
                blockchain,
            )
        };
        let mining_address = client.get_new_segwit_confidential_address().await.unwrap();

        let have_asset_id_alice = client.get_bitcoin_asset_id().await.unwrap();
        let have_asset_id_bob = client.issueasset(100_000.0, 0.0, true).await.unwrap().asset;

        let rate_service = fixed_rate::Service::new();
        let redeem_amount_bob = Amount::ONE_BTC;

        let (
            fund_address_alice,
            fund_sk_alice,
            _fund_pk_alice,
            fund_blinding_sk_alice,
            _fund_blinding_pk_alice,
        ) = make_confidential_address();

        let fund_alice_txid = client
            .send_asset_to_address(
                &fund_address_alice,
                redeem_amount_bob + Amount::ONE_BTC,
                Some(have_asset_id_alice),
            )
            .await
            .unwrap();
        client.generatetoaddress(1, &mining_address).await.unwrap();

        let input_alice = extract_input(
            &client.get_raw_transaction(fund_alice_txid).await.unwrap(),
            fund_address_alice,
        )
        .unwrap();

        let (
            final_address_alice,
            _final_sk_alice,
            _final_pk_alice,
            _final_blinding_sk_alice,
            _final_blinding_pk_alice,
        ) = make_confidential_address();

        // move issued asset to wallet address
        let address = client.get_new_segwit_confidential_address().await.unwrap();
        let _txid = client
            .send_asset_to_address(
                &address,
                Amount::from_btc(10.0).unwrap(),
                Some(have_asset_id_bob),
            )
            .await
            .unwrap();
        client.generatetoaddress(1, &mining_address).await.unwrap();

        let mut bob = Bobtimus {
            rng: &mut thread_rng(),
            rate_service,
            secp: Secp256k1::new(),
            elementsd: client.clone(),
            btc_asset_id: have_asset_id_alice,
            usdt_asset_id: have_asset_id_bob,
            db,
            lender_states: HashMap::new(),
        };

        let transaction = bob
            .handle_create_sell_swap(CreateSwapPayload {
                alice_inputs: vec![AliceInput {
                    outpoint: input_alice.0,
                    blinding_key: fund_blinding_sk_alice,
                }],
                address: final_address_alice,
                amount: redeem_amount_bob.as_sat(),
            })
            .await
            .unwrap();

        let transaction = swap::alice_finalize_transaction(transaction, {
            let value = input_alice.1.value;
            move |mut tx| async move {
                let input_index = tx
                    .input
                    .iter()
                    .position(|txin| fund_alice_txid == txin.previous_output.txid)
                    .context("transaction does not contain input")?;
                let mut cache = SigHashCache::new(&tx);

                tx.input[input_index].witness.script_witness =
                    sign_with_key(&SECP256K1, &mut cache, input_index, &fund_sk_alice, value);

                Ok(tx)
            }
        })
        .await
        .unwrap();

        let _txid = client.send_raw_transaction(&transaction).await.unwrap();
        let _txid = client.generatetoaddress(1, &mining_address).await.unwrap();

        let utxos = client
            .listunspent(
                None,
                None,
                None,
                None,
                Some(ListUnspentOptions {
                    asset: Some(have_asset_id_alice),
                    ..Default::default()
                }),
            )
            .await
            .unwrap();

        let fee = transaction.fee_in(have_asset_id_bob);
        assert!(utxos.iter().any(|utxo| (utxo.amount
            - (redeem_amount_bob.as_btc() - Amount::from_sat(fee).as_btc()))
        .abs()
            < f64::EPSILON
            && utxo.spendable));
    }

    #[tokio::test]
    async fn test_handle_btc_buy_swap_request() {
        let db = Sqlite::new_ephemeral_db().expect("A ephemeral db");

        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into()).unwrap(),
                blockchain,
            )
        };
        let mining_address = client.get_new_segwit_confidential_address().await.unwrap();

        let have_asset_id_alice = client.issueasset(100_000.0, 0.0, true).await.unwrap().asset;
        let have_asset_id_bob = client.get_bitcoin_asset_id().await.unwrap();

        let rate_service = fixed_rate::Service::new();
        let redeem_amount_bob = LiquidUsdt::from_str_in_dollar("20000.0").unwrap();

        let (
            fund_address_alice,
            fund_sk_alice,
            _fund_pk_alice,
            fund_blinding_sk_alice,
            _fund_blinding_pk_alice,
        ) = make_confidential_address();

        let fund_alice_txid = client
            .send_asset_to_address(
                &fund_address_alice,
                Amount::from_btc(10_000.0).unwrap() + redeem_amount_bob.into(),
                Some(have_asset_id_alice),
            )
            .await
            .unwrap();
        client.generatetoaddress(1, &mining_address).await.unwrap();

        let input_alice = extract_input(
            &client.get_raw_transaction(fund_alice_txid).await.unwrap(),
            fund_address_alice,
        )
        .unwrap();

        let (
            final_address_alice,
            _final_sk_alice,
            _final_pk_alice,
            _final_blinding_sk_alice,
            _final_blinding_pk_alice,
        ) = make_confidential_address();

        let mut bob = Bobtimus {
            rng: &mut thread_rng(),
            rate_service,
            secp: Secp256k1::new(),
            elementsd: client.clone(),
            btc_asset_id: have_asset_id_bob,
            usdt_asset_id: have_asset_id_alice,
            db,
            lender_states: HashMap::new(),
        };

        let transaction = bob
            .handle_create_buy_swap(CreateSwapPayload {
                alice_inputs: vec![AliceInput {
                    outpoint: input_alice.0,
                    blinding_key: fund_blinding_sk_alice,
                }],
                address: final_address_alice,
                amount: redeem_amount_bob.as_satodollar(),
            })
            .await
            .unwrap();

        let transaction = swap::alice_finalize_transaction(transaction, {
            let value = input_alice.1.value;
            move |mut tx| async move {
                let input_index = tx
                    .input
                    .iter()
                    .position(|txin| fund_alice_txid == txin.previous_output.txid)
                    .context("transaction does not contain input")?;
                let mut cache = SigHashCache::new(&tx);

                tx.input[input_index].witness.script_witness =
                    sign_with_key(&SECP256K1, &mut cache, input_index, &fund_sk_alice, value);

                Ok(tx)
            }
        })
        .await
        .unwrap();

        let _txid = client.send_raw_transaction(&transaction).await.unwrap();
        let _txid = client.generatetoaddress(1, &mining_address).await.unwrap();

        let utxos = client
            .listunspent(
                None,
                None,
                None,
                None,
                Some(ListUnspentOptions {
                    asset: Some(have_asset_id_alice),
                    ..Default::default()
                }),
            )
            .await
            .unwrap();

        assert!(utxos.iter().any(
            |utxo| (utxo.amount - Amount::from(redeem_amount_bob).as_btc()).abs() < f64::EPSILON
                && utxo.spendable
        ));
    }

    fn extract_input(tx: &Transaction, address: Address) -> Result<(OutPoint, TxOut)> {
        let vout = tx
            .output
            .iter()
            .position(|output| output.script_pubkey == address.script_pubkey())
            .context("Tx doesn't pay to address")?;

        let outpoint = OutPoint {
            txid: tx.txid(),
            vout: vout as u32,
        };
        let txout = tx.output[vout].clone();
        Ok((outpoint, txout))
    }

    fn make_keypair() -> (SecretKey, PublicKey) {
        let sk = SecretKey::new(&mut thread_rng());
        let pk = PublicKey::from_private_key(
            &SECP256K1,
            &PrivateKey {
                compressed: true,
                network: Network::Regtest,
                key: sk,
            },
        );

        (sk, pk)
    }

    fn make_confidential_address() -> (Address, SecretKey, PublicKey, SecretKey, PublicKey) {
        let (sk, pk) = make_keypair();
        let (blinding_sk, blinding_pk) = make_keypair();

        (
            Address::p2wpkh(&pk, Some(blinding_pk.key), &AddressParams::ELEMENTS),
            sk,
            pk,
            blinding_sk,
            blinding_pk,
        )
    }
}
