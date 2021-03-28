use crate::stack_simulator::simulate;
use anyhow::{bail, Context, Result};
use elements::{
    bitcoin::{util::psbt::serialize::Serialize, Amount, Network, PrivateKey, PublicKey},
    confidential::{Asset, AssetBlindingFactor, Value, ValueBlindingFactor},
    encode::Encodable,
    hashes::{sha256d, Hash},
    opcodes::all::*,
    script::Builder,
    secp256k1::{
        rand::{thread_rng, CryptoRng, RngCore},
        Secp256k1, SecretKey, Signature, Signing, Verification, SECP256K1,
    },
    sighash::SigHashCache,
    Address, AddressParams, AssetId, AssetIssuance, ConfidentialTxOut, OutPoint, Script,
    SigHashType, Transaction, TxIn, TxInWitness, TxOut, UnblindedTxOut,
};
use secp256k1_zkp::{SurjectionProof, Tag};
use std::future::Future;

/// These constants have been reverse engineered through the following transactions:
///
/// https://blockstream.info/liquid/tx/a17f4063b3a5fdf46a7012c82390a337e9a0f921933dccfb8a40241b828702f2
/// https://blockstream.info/liquid/tx/d12ff4e851816908810c7abc839dd5da2c54ad24b4b52800187bee47df96dd5c
/// https://blockstream.info/liquid/tx/47e60a3bc5beed45a2cf9fb7a8d8969bab4121df98b0034fb0d44f6ed2d60c7d
///
/// This gives us the following set of linear equations:
///
/// - 1 in, 1 out, 1 fee = 1332
/// - 1 in, 2 out, 1 fee = 2516
/// - 2 in, 2 out, 1 fee = 2623
///
/// Which we can solve using wolfram alpha: https://www.wolframalpha.com/input/?i=1x+%2B+1y+%2B+1z+%3D+1332%2C+1x+%2B+2y+%2B+1z+%3D+2516%2C+2x+%2B+2y+%2B+1z+%3D+2623
pub mod avg_vbytes {
    pub const INPUT: u64 = 107;
    pub const OUTPUT: u64 = 1184;
    pub const FEE: u64 = 41;
}

/// Estimate the virtual size of a transaction based on the number of inputs and outputs.
pub fn estimate_virtual_size(number_of_inputs: u64, number_of_outputs: u64) -> u64 {
    number_of_inputs * avg_vbytes::INPUT + number_of_outputs * avg_vbytes::OUTPUT + avg_vbytes::FEE
}

#[cfg(test)]
mod protocol_tests;
mod stack_simulator;

pub struct LoanRequest {
    collateral_amount: Amount,
    collateral_inputs: Vec<Input>,
    fee_sats_per_vbyte: Amount,
    borrower_pk: PublicKey,
    timelock: u64,
    borrower_address: Address,
}

pub struct LoanResponse {
    transaction: Transaction,
    _principal_amount: Amount,
    lender_pk: PublicKey,
    repayment_collateral_input: Input,
    repayment_collateral_abf: AssetBlindingFactor,
    repayment_collateral_vbf: ValueBlindingFactor,
    timelock: u64,
    repayment_principal_output: TxOut,
}

pub struct Borrower0 {
    keypair: (SecretKey, PublicKey),
    address: Address,
    address_blinding_sk: SecretKey,
    collateral_amount: Amount,
    collateral_inputs: Vec<Input>,
    fee_sats_per_vbyte: Amount,
    timelock: u64,
    bitcoin_asset_id: AssetId,
    usdt_asset_id: AssetId,
}

impl Borrower0 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address: Address,
        address_blinding_sk: SecretKey,
        collateral_amount: Amount,
        collateral_inputs: Vec<Input>,
        fee_sats_per_vbyte: Amount,
        timelock: u64,
        bitcoin_asset_id: AssetId,
        usdt_asset_id: AssetId,
    ) -> Result<Self> {
        let keypair = make_keypair();

        Ok(Self {
            keypair,
            address,
            address_blinding_sk,
            collateral_amount,
            collateral_inputs,
            fee_sats_per_vbyte,
            timelock,
            bitcoin_asset_id,
            usdt_asset_id,
        })
    }

    pub fn loan_request(&self) -> LoanRequest {
        LoanRequest {
            collateral_amount: self.collateral_amount,
            collateral_inputs: self.collateral_inputs.clone(),
            fee_sats_per_vbyte: self.fee_sats_per_vbyte,
            borrower_pk: self.keypair.1,
            timelock: self.timelock,
            borrower_address: self.address.clone(),
        }
    }

    pub fn interpret<C>(self, secp: &Secp256k1<C>, loan_response: LoanResponse) -> Result<Borrower1>
    where
        C: Signing + Verification,
    {
        let transaction = loan_response.transaction;

        let principal_tx_out_amount = transaction
            .output
            .iter()
            .find_map(|out| match out.to_confidential() {
                Some(conf) => {
                    let unblinded_out = conf.unblind(secp, self.address_blinding_sk).ok()?;
                    let predicate = unblinded_out.asset == self.usdt_asset_id
                        && conf.script_pubkey == self.address.script_pubkey();

                    predicate.then(|| Amount::from_sat(unblinded_out.value))
                }
                None => None,
            })
            .context("no principal txout")?;

        // TODO: Verify repayment collateral input to ensure that the
        // lender agrees with the repayment condition

        let collateral_script = loan_contract(
            self.keypair.1,
            loan_response.lender_pk,
            loan_response.timelock,
            loan_response.repayment_principal_output.clone(),
        )?;

        let collateral_address = Address::p2wsh(&collateral_script, None, &AddressParams::ELEMENTS);
        let collateral_script_pubkey = collateral_address.script_pubkey();
        let collateral_blinding_sk = loan_response.repayment_collateral_input.blinding_key;
        transaction
            .output
            .iter()
            .find_map(|out| match out.to_confidential() {
                Some(conf) => {
                    let unblinded_out = conf.unblind(secp, collateral_blinding_sk).ok()?;
                    let predicate = unblinded_out.asset == self.bitcoin_asset_id
                        && unblinded_out.value == self.collateral_amount.as_sat()
                        && out.script_pubkey == collateral_script_pubkey;

                    predicate.then(|| out)
                }
                None => None,
            })
            .context("no collateral txout")?;

        let collateral_input_amount = self
            .collateral_inputs
            .iter()
            .map(|input| input.clone().into_unblinded_input(secp))
            .try_fold(0, |sum, input| {
                input.map(|input| sum + input.unblinded.value).ok()
            })
            .context("could not sum collateral inputs")?;
        let tx_fee = Amount::from_sat(
            estimate_virtual_size(transaction.input.len() as u64, 4)
                * self.fee_sats_per_vbyte.as_sat(),
        );
        let collateral_change_amount = Amount::from_sat(collateral_input_amount)
            .checked_sub(self.collateral_amount)
            .map(|a| a.checked_sub(tx_fee))
            .flatten()
            .with_context(|| {
                format!(
                    "cannot pay for output {} and fee {} with input {}",
                    self.collateral_amount, tx_fee, collateral_input_amount,
                )
            })?;

        transaction
            .output
            .iter()
            .find_map(|out| match out.to_confidential() {
                Some(conf) => {
                    let unblinded_out = conf.unblind(secp, self.address_blinding_sk).ok()?;
                    let predicate = unblinded_out.asset == self.bitcoin_asset_id
                        && unblinded_out.value == collateral_change_amount.as_sat()
                        && out.script_pubkey == self.address.script_pubkey();

                    predicate.then(|| out)
                }
                None => None,
            })
            .context("no collateral change txout")?;

        Ok(Borrower1 {
            keypair: self.keypair,
            loan_transaction: transaction,
            collateral_amount: self.collateral_amount,
            collateral_script,
            principal_tx_out_amount,
            address: self.address.clone(),
            repayment_collateral_input: loan_response.repayment_collateral_input,
            repayment_collateral_abf: loan_response.repayment_collateral_abf,
            repayment_collateral_vbf: loan_response.repayment_collateral_vbf,
            bitcoin_asset_id: self.bitcoin_asset_id,
            usdt_asset_id: self.usdt_asset_id,
            repayment_principal_output: loan_response.repayment_principal_output,
        })
    }
}

pub struct Borrower1 {
    keypair: (SecretKey, PublicKey),
    loan_transaction: Transaction,
    collateral_amount: Amount,
    collateral_script: Script,
    principal_tx_out_amount: Amount,
    address: Address,
    repayment_collateral_input: Input,
    repayment_collateral_abf: AssetBlindingFactor,
    repayment_collateral_vbf: ValueBlindingFactor,
    bitcoin_asset_id: AssetId,
    usdt_asset_id: AssetId,
    repayment_principal_output: TxOut,
}

impl Borrower1 {
    pub async fn sign<S, F>(&self, signer: S) -> Result<Transaction>
    where
        S: FnOnce(Transaction) -> F,
        F: Future<Output = Result<Transaction>>,
    {
        signer(self.loan_transaction.clone()).await
    }

    pub async fn loan_repayment_transaction<R, C, CS, CF, SI, SF>(
        &self,
        rng: &mut R,
        secp: &Secp256k1<C>,
        coin_selector: CS,
        signer: SI,
        tx_fee: Amount,
    ) -> Result<Transaction>
    where
        R: RngCore + CryptoRng,
        C: Verification + Signing,
        CS: FnOnce(Amount, AssetId) -> CF,
        CF: Future<Output = Result<Vec<Input>>>,
        SI: FnOnce(Transaction, u32, Address, Value) -> SF,
        SF: Future<Output = Result<Transaction>>,
    {
        let repayment_amount = self.principal_tx_out_amount;

        // construct collateral input
        let collateral_input = self
            .repayment_collateral_input
            .clone()
            .into_unblinded_input(secp)
            .context("could not unblind repayment collateral input")?;
        let principal_inputs = coin_selector(repayment_amount, self.usdt_asset_id).await?;

        let unblinded_principal_inputs = principal_inputs
            .clone()
            .into_iter()
            .map(|input| input.into_unblinded_input(secp))
            .collect::<Result<Vec<_>>>()?;

        let inputs = {
            let mut borrower_inputs = unblinded_principal_inputs
                .iter()
                .map(|input| {
                    (
                        input.unblinded.asset,
                        input.unblinded.value,
                        input.confidential.asset,
                        input.unblinded.asset_blinding_factor,
                        input.unblinded.value_blinding_factor,
                    )
                })
                .collect::<Vec<_>>();
            borrower_inputs.push((
                collateral_input.unblinded.asset,
                collateral_input.unblinded.value,
                collateral_input.confidential.asset,
                collateral_input.unblinded.asset_blinding_factor,
                collateral_input.unblinded.value_blinding_factor,
            ));

            borrower_inputs
        };

        let mut repayment_principal_output = self.repayment_principal_output.clone();
        repayment_principal_output.witness.surjection_proof = SurjectionProof::new(
            secp,
            rng,
            Tag::from(self.usdt_asset_id.into_inner().0),
            self.repayment_collateral_abf.into_inner(),
            &inputs
                .iter()
                .map(|(id, _, asset, abf, _)| {
                    (*asset, Tag::from(id.into_inner().0), abf.into_inner())
                })
                .collect::<Vec<_>>(),
        )?
        .serialize();

        let principal_input_amount = unblinded_principal_inputs
            .iter()
            .fold(0, |acc, input| acc + input.unblinded.value);
        let change_amount = Amount::from_sat(principal_input_amount)
            .checked_sub(repayment_amount)
            .with_context(|| {
                format!(
                    "cannot pay for output {} with input {}",
                    repayment_amount, principal_input_amount,
                )
            })?;

        let mut outputs = vec![(
            repayment_amount.as_sat(),
            self.repayment_collateral_abf,
            self.repayment_collateral_vbf,
        )];

        let change_output = match change_amount {
            Amount::ZERO => None,
            _ => {
                let (output, abf, vbf) = TxOut::new_not_last_confidential(
                    rng,
                    secp,
                    change_amount.as_sat(),
                    self.address.clone(),
                    self.usdt_asset_id,
                    &inputs,
                )
                .context("Change output creation failed")?;

                outputs.push((change_amount.as_sat(), abf, vbf));

                Some(output)
            }
        };

        let collateral_output = TxOut::new_last_confidential(
            rng,
            secp,
            (self.collateral_amount - tx_fee).as_sat(),
            self.address.clone(),
            self.bitcoin_asset_id,
            &inputs,
            &outputs,
        )
        .context("Creation of collateral output failed")?;

        let tx_fee_output = TxOut::new_fee(tx_fee.as_sat(), self.bitcoin_asset_id);

        let mut tx_ins: Vec<TxIn> = unblinded_principal_inputs
            .into_iter()
            .map(|input| input.tx_in)
            .collect();
        tx_ins.push(collateral_input.tx_in);

        let mut tx_outs = vec![repayment_principal_output.clone()];
        if let Some(change_output) = change_output {
            tx_outs.push(change_output)
        }
        tx_outs.push(collateral_output);
        tx_outs.push(tx_fee_output);

        let mut tx = Transaction {
            version: 2,
            lock_time: 0,
            input: tx_ins,
            output: tx_outs,
        };

        // fulfill collateral input covenant script
        {
            let sighash = SigHashCache::new(&tx).segwitv0_sighash(
                1,
                &self.collateral_script.clone(),
                self.repayment_collateral_input.original_tx_out.value,
                SigHashType::All,
            );

            let sig = SECP256K1.sign(
                &elements::secp256k1::Message::from(sighash),
                &self.keypair.0,
            );

            let script_witness = RepaymentWitnessStack::new(
                sig,
                self.keypair.1,
                self.repayment_collateral_input.original_tx_out.value,
                &tx,
                self.collateral_script.clone(),
            )
            .unwrap()
            .serialise()
            .unwrap();

            simulate(self.collateral_script.clone(), script_witness.clone()).unwrap();

            tx.input[1].witness = TxInWitness {
                amount_rangeproof: vec![],
                inflation_keys_rangeproof: vec![],
                script_witness,
                pegin_witness: vec![],
            };
        };

        // TODO: Sign more than one input if necessary
        // sign repayment input of the principal amount
        let tx = {
            let script_pubkey = principal_inputs[0].original_tx_out.script_pubkey.clone();
            let blinder_sk = principal_inputs[0].blinding_key;
            let blinder_pk = secp256k1_zkp::PublicKey::from_secret_key(secp, &blinder_sk);
            let address =
                Address::from_script(&script_pubkey, Some(blinder_pk), &AddressParams::ELEMENTS)
                    .unwrap();
            let value = principal_inputs[0].original_tx_out.value;
            signer(tx, 0, address, value).await?
        };

        Ok(tx)
    }
}

pub struct Lender0 {
    keypair: (SecretKey, PublicKey),
    principal_inputs: Vec<UnblindedInput>,
    address: Address,
    bitcoin_asset_id: AssetId,
    usdt_asset_id: AssetId,
}

impl Lender0 {
    pub fn new<C>(
        secp: &Secp256k1<C>,
        bitcoin_asset_id: AssetId,
        usdt_asset_id: AssetId,
        // TODO: Here we assume that the wallet is giving us _all_ the
        // inputs available. It would be better to coin-select these
        // as soon as we know the principal amount after receiving the
        // loan request
        principal_inputs: Vec<Input>,
        address: Address,
    ) -> Result<Self>
    where
        C: Verification,
    {
        let keypair = make_keypair();

        let principal_inputs = principal_inputs
            .into_iter()
            .map(|input| input.into_unblinded_input(secp))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            bitcoin_asset_id,
            keypair,
            address,
            usdt_asset_id,
            principal_inputs,
        })
    }

    pub fn interpret<R, C>(
        self,
        rng: &mut R,
        secp: &Secp256k1<C>,
        loan_request: LoanRequest,
    ) -> Result<Lender1>
    where
        R: RngCore + CryptoRng,
        C: Verification + Signing,
    {
        let principal_amount = Lender0::calc_principal_amount(&loan_request);
        let collateral_inputs = loan_request
            .collateral_inputs
            .into_iter()
            .map(|input| input.into_unblinded_input(secp))
            .collect::<Result<Vec<_>>>()?;

        let borrower_inputs = collateral_inputs.iter().map(|input| {
            (
                input.unblinded.asset,
                input.unblinded.value,
                input.confidential.asset,
                input.unblinded.asset_blinding_factor,
                input.unblinded.value_blinding_factor,
            )
        });
        let lender_inputs = self.principal_inputs.iter().map(|input| {
            (
                input.unblinded.asset,
                input.unblinded.value,
                input.confidential.asset,
                input.unblinded.asset_blinding_factor,
                input.unblinded.value_blinding_factor,
            )
        });

        let inputs = borrower_inputs.chain(lender_inputs).collect::<Vec<_>>();

        let collateral_input_amount = collateral_inputs
            .iter()
            .fold(0, |sum, input| sum + input.unblinded.value);

        let collateral_amount = loan_request.collateral_amount;

        let (repayment_principal_output, repayment_collateral_abf, repayment_collateral_vbf) = {
            let dummy_asset = self.usdt_asset_id;
            let dummy_abf = AssetBlindingFactor::random(rng);
            let dummy_generator = Asset::new_confidential(secp, dummy_asset, dummy_abf)
                .commitment()
                .expect("confidential");
            let dummy_amount = principal_amount.as_sat();
            let dummy_vbf = ValueBlindingFactor::random(rng);
            let dummy_inputs = &[(
                dummy_asset,
                dummy_amount,
                dummy_generator,
                dummy_abf,
                dummy_vbf,
            )];

            TxOut::new_not_last_confidential(
                rng,
                secp,
                principal_amount.as_sat(),
                self.address.clone(),
                self.usdt_asset_id,
                dummy_inputs,
            )?
        };

        let (_, lender_pk) = self.keypair;
        let collateral_script = loan_contract(
            loan_request.borrower_pk,
            lender_pk,
            loan_request.timelock,
            repayment_principal_output.clone(),
        )?;

        let (collateral_blinding_sk, collateral_blinding_pk) = make_keypair();
        let collateral_address = Address::p2wsh(
            &collateral_script,
            Some(collateral_blinding_pk.key),
            &AddressParams::ELEMENTS,
        );
        let (collateral_tx_out, abf_collateral, vbf_collateral) = TxOut::new_not_last_confidential(
            rng,
            secp,
            collateral_amount.as_sat(),
            collateral_address,
            self.bitcoin_asset_id,
            &inputs,
        )
        .context("could not construct collateral txout")?;

        let (principal_tx_out, abf_principal, vbf_principal) = TxOut::new_not_last_confidential(
            rng,
            secp,
            principal_amount.as_sat(),
            loan_request.borrower_address.clone(),
            self.usdt_asset_id,
            &inputs,
        )
        .context("could not construct principal txout")?;

        let principal_input_amount = self
            .principal_inputs
            .iter()
            .fold(0, |sum, input| sum + input.unblinded.value);
        let principal_change_amount = Amount::from_sat(principal_input_amount) - principal_amount;
        let (principal_change_tx_out, abf_principal_change, vbf_principal_change) =
            TxOut::new_not_last_confidential(
                rng,
                secp,
                principal_change_amount.as_sat(),
                self.address.clone(),
                self.usdt_asset_id,
                &inputs,
            )
            .context("could not construct principal change txout")?;

        let not_last_confidential_outputs = [
            (collateral_amount.as_sat(), abf_collateral, vbf_collateral),
            (principal_amount.as_sat(), abf_principal, vbf_principal),
            (
                principal_change_amount.as_sat(),
                abf_principal_change,
                vbf_principal_change,
            ),
        ];

        let tx_fee = Amount::from_sat(
            estimate_virtual_size(inputs.len() as u64, 4)
                * loan_request.fee_sats_per_vbyte.as_sat(),
        );
        let collateral_change_amount = Amount::from_sat(collateral_input_amount)
            .checked_sub(collateral_amount)
            .map(|a| a.checked_sub(tx_fee))
            .flatten()
            .with_context(|| {
                format!(
                    "cannot pay for output {} and fee {} with input {}",
                    collateral_amount, tx_fee, collateral_input_amount,
                )
            })?;
        let collateral_change_tx_out = TxOut::new_last_confidential(
            rng,
            secp,
            collateral_change_amount.as_sat(),
            loan_request.borrower_address,
            self.bitcoin_asset_id,
            &inputs,
            &not_last_confidential_outputs,
        )
        .context("Creation of collateral change output failed")?;

        let tx_ins = {
            let borrower_inputs = collateral_inputs.iter().map(|input| input.tx_in.clone());
            let lender_inputs = self
                .principal_inputs
                .iter()
                .map(|input| input.tx_in.clone());
            borrower_inputs.chain(lender_inputs).collect::<Vec<_>>()
        };

        let tx_fee_tx_out = TxOut::new_fee(tx_fee.as_sat(), self.bitcoin_asset_id);

        let loan_transaction = Transaction {
            version: 2,
            lock_time: 0,
            input: tx_ins,
            output: vec![
                collateral_tx_out.clone(),
                principal_tx_out,
                principal_change_tx_out,
                collateral_change_tx_out,
                tx_fee_tx_out,
            ],
        };

        let repayment_collateral_input = Input {
            tx_in: TxIn {
                previous_output: OutPoint {
                    txid: loan_transaction.txid(),
                    vout: 0,
                },
                is_pegin: false,
                has_issuance: false,
                script_sig: Script::new(),
                sequence: 0,
                asset_issuance: AssetIssuance::default(),
                witness: TxInWitness::default(),
            },
            original_tx_out: collateral_tx_out,
            blinding_key: collateral_blinding_sk,
        };

        Ok(Lender1 {
            keypair: self.keypair,
            address: self.address,
            timelock: loan_request.timelock,
            loan_transaction,
            collateral_script,
            collateral_amount: loan_request.collateral_amount,
            principal_amount,
            repayment_collateral_input,
            repayment_collateral_abf,
            repayment_collateral_vbf,
            bitcoin_asset_id: self.bitcoin_asset_id,
            repayment_principal_output,
        })
    }

    fn calc_principal_amount(loan_request: &LoanRequest) -> Amount {
        Amount::from_sat(loan_request.collateral_amount.as_sat() / 2)
    }
}

pub struct Lender1 {
    keypair: (SecretKey, PublicKey),
    address: Address,
    timelock: u64,
    loan_transaction: Transaction,
    collateral_script: Script,
    collateral_amount: Amount,
    principal_amount: Amount,
    repayment_collateral_input: Input,
    repayment_collateral_abf: AssetBlindingFactor,
    repayment_collateral_vbf: ValueBlindingFactor,
    bitcoin_asset_id: AssetId,
    repayment_principal_output: TxOut,
}

impl Lender1 {
    pub fn loan_response(&self) -> LoanResponse {
        LoanResponse {
            transaction: self.loan_transaction.clone(),
            _principal_amount: self.principal_amount,
            lender_pk: self.keypair.1,
            repayment_collateral_input: self.repayment_collateral_input.clone(),
            repayment_collateral_abf: self.repayment_collateral_abf,
            repayment_collateral_vbf: self.repayment_collateral_vbf,
            timelock: self.timelock,
            repayment_principal_output: self.repayment_principal_output.clone(),
        }
    }

    pub async fn finalise_loan<S, F>(
        &self,
        loan_transaction: Transaction,
        signer: S,
    ) -> Result<Transaction>
    where
        S: FnOnce(Transaction) -> F,
        F: Future<Output = Result<Transaction>>,
    {
        if self.loan_transaction.txid() != loan_transaction.txid() {
            bail!("wrong loan transaction")
        }

        signer(loan_transaction).await
    }

    pub fn liquidation_transaction<R, C>(
        &self,
        rng: &mut R,
        secp: &Secp256k1<C>,
        tx_fee: Amount,
    ) -> Result<Transaction>
    where
        R: RngCore + CryptoRng,
        C: Verification + Signing,
    {
        // construct collateral input
        let collateral_input = self
            .repayment_collateral_input
            .clone()
            .into_unblinded_input(secp)
            .context("could not unblind repayment collateral input")?;

        let inputs = vec![(
            collateral_input.unblinded.asset,
            collateral_input.unblinded.value,
            collateral_input.confidential.asset,
            collateral_input.unblinded.asset_blinding_factor,
            collateral_input.unblinded.value_blinding_factor,
        )];

        let collateral_output = TxOut::new_last_confidential(
            rng,
            secp,
            (self.collateral_amount - tx_fee).as_sat(),
            self.address.clone(),
            self.bitcoin_asset_id,
            &inputs,
            &[],
        )
        .context("Creation of collateral output failed")?;

        let tx_fee_output = TxOut::new_fee(tx_fee.as_sat(), self.bitcoin_asset_id);

        let tx_ins = vec![collateral_input.tx_in];
        let tx_outs = vec![collateral_output, tx_fee_output];

        let mut liquidation_transaction = Transaction {
            version: 2,
            lock_time: 0,
            input: tx_ins,
            output: tx_outs,
        };

        // fulfill collateral input covenant script to liquidate the position
        let sighash = SigHashCache::new(&liquidation_transaction).segwitv0_sighash(
            0,
            &self.collateral_script.clone(),
            self.repayment_collateral_input.original_tx_out.value,
            SigHashType::All,
        );

        let sig = SECP256K1.sign(
            &elements::secp256k1::Message::from(sighash),
            &self.keypair.0,
        );

        let mut sig = sig.serialize_der().to_vec();
        sig.push(SigHashType::All as u8);
        let if_flag = vec![];

        liquidation_transaction.input[0].witness = TxInWitness {
            amount_rangeproof: vec![],
            inflation_keys_rangeproof: vec![],
            script_witness: vec![sig, if_flag, self.collateral_script.to_bytes()],
            pegin_witness: vec![],
        };

        Ok(liquidation_transaction)
    }
}

fn loan_contract(
    borrower_pk: PublicKey,
    lender_pk: PublicKey,
    timelock: u64,
    repayment_output: TxOut,
) -> Result<Script> {
    let mut repayment_output_bytes = Vec::new();
    repayment_output.consensus_encode(&mut repayment_output_bytes)?;

    Ok(Builder::new()
        .push_opcode(OP_IF)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_slice(repayment_output_bytes.as_slice())
        .push_opcode(OP_SWAP)
        .push_opcode(OP_CAT)
        .push_opcode(OP_HASH256)
        .push_opcode(OP_DEPTH)
        .push_opcode(OP_1SUB)
        .push_opcode(OP_PICK)
        .push_opcode(OP_PUSHNUM_1)
        .push_opcode(OP_CAT)
        .push_slice(&borrower_pk.serialize())
        .push_opcode(OP_CHECKSIGVERIFY)
        .push_opcode(OP_TOALTSTACK)
        .push_opcode(OP_CAT)
        .push_opcode(OP_FROMALTSTACK)
        .push_opcode(OP_SWAP)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_SHA256)
        .push_opcode(OP_SWAP)
        .push_opcode(OP_CHECKSIGFROMSTACK)
        .push_opcode(OP_ELSE)
        .push_int(timelock as i64)
        .push_opcode(OP_CLTV)
        .push_opcode(OP_DROP)
        .push_slice(&lender_pk.serialize())
        .push_opcode(OP_CHECKSIG)
        .push_opcode(OP_ENDIF)
        .into_script())
}

struct RepaymentWitnessStack {
    sig: Signature,
    pk: PublicKey,
    tx_version: u32,
    hash_prev_out: sha256d::Hash,
    hash_sequence: sha256d::Hash,
    hash_issuances: sha256d::Hash,
    input: InputData,
    other_outputs: Vec<TxOut>,
    lock_time: u32,
    sighash_type: SigHashType,
}

struct InputData {
    previous_output: OutPoint,
    script: Script,
    value: Value,
    sequence: u32,
}

impl RepaymentWitnessStack {
    fn new(
        sig: Signature,
        pk: PublicKey,
        collateral_value: Value,
        tx: &Transaction,
        script: Script,
    ) -> Result<Self> {
        let tx_version = tx.version;

        let hash_prev_out = {
            let mut enc = sha256d::Hash::engine();
            for txin in tx.input.iter() {
                txin.previous_output.consensus_encode(&mut enc)?;
            }

            sha256d::Hash::from_engine(enc)
        };

        let hash_sequence = {
            let mut enc = sha256d::Hash::engine();

            for txin in tx.input.iter() {
                txin.sequence.consensus_encode(&mut enc)?;
            }
            sha256d::Hash::from_engine(enc)
        };

        let hash_issuances = {
            let mut enc = sha256d::Hash::engine();
            for txin in tx.input.iter() {
                if txin.has_issuance() {
                    txin.asset_issuance.consensus_encode(&mut enc)?;
                } else {
                    0u8.consensus_encode(&mut enc)?;
                }
            }
            sha256d::Hash::from_engine(enc)
        };

        let input = {
            let input = &tx.input[1];
            InputData {
                previous_output: input.previous_output,
                script,
                value: collateral_value,
                sequence: input.sequence,
            }
        };

        let other_outputs = tx.output[1..].to_vec();

        let lock_time = tx.lock_time;

        let sighash_type = SigHashType::All;

        Ok(Self {
            sig,
            pk,
            tx_version,
            hash_prev_out,
            hash_sequence,
            hash_issuances,
            input,
            other_outputs,
            lock_time,
            sighash_type,
        })
    }

    // Items on the witness stack are limited to 80 bytes, so we have
    // to split things all around the place e.g. the script in the
    // input that we sign and the "other" outputs
    fn serialise(&self) -> anyhow::Result<Vec<Vec<u8>>> {
        let if_flag = vec![0x01];

        let sig = self.sig.serialize_der().to_vec();

        let pk = self.pk.serialize().to_vec();

        let tx_version = {
            let mut writer = Vec::new();
            self.tx_version.consensus_encode(&mut writer)?;
            writer
        };

        // input specific values
        let (previous_out, script_0, script_1, script_2, value, sequence) = {
            let InputData {
                previous_output,
                script,
                value,
                sequence,
            } = &self.input;

            let third = script.len() / 3;

            (
                {
                    let mut writer = Vec::new();
                    previous_output.consensus_encode(&mut writer)?;
                    writer
                },
                {
                    let mut writer = Vec::new();
                    script.consensus_encode(&mut writer)?;
                    writer[..third].to_vec()
                },
                {
                    let mut writer = Vec::new();
                    script.consensus_encode(&mut writer)?;
                    writer[third..2 * third].to_vec()
                },
                {
                    let mut writer = Vec::new();
                    script.consensus_encode(&mut writer)?;
                    writer[2 * third..].to_vec()
                },
                {
                    let mut writer = Vec::new();
                    value.consensus_encode(&mut writer)?;
                    writer
                },
                {
                    let mut writer = Vec::new();
                    sequence.consensus_encode(&mut writer)?;
                    writer
                },
            )
        };

        // hashoutputs (only supporting SigHashType::All)
        let other_outputs = {
            let mut other_outputs = vec![];

            if self.other_outputs.len() < 2 {
                bail!("insufficient outputs");
            }

            if self.other_outputs.len() > 3 {
                bail!("too many outputs");
            }

            for txout in self.other_outputs.iter() {
                let mut output = Vec::new();
                txout.consensus_encode(&mut output)?;

                let middle = output.len() / 2;
                other_outputs.push(output[..middle].to_vec());
                other_outputs.push(output[middle..].to_vec());
            }

            // fill in space for missing principal change output
            if other_outputs.len() == 4 {
                other_outputs.push(vec![]);
                other_outputs.push(vec![]);
            }

            other_outputs
        };

        let lock_time = {
            let mut writer = Vec::new();
            self.lock_time.consensus_encode(&mut writer)?;
            writer
        };

        let sighash_type = {
            let mut writer = Vec::new();
            self.sighash_type.as_u32().consensus_encode(&mut writer)?;
            writer
        };

        Ok(vec![
            sig,
            pk,
            tx_version,
            self.hash_prev_out.to_vec(),
            self.hash_sequence.to_vec(),
            self.hash_issuances.to_vec(),
            previous_out,
            script_0,
            script_1,
            script_2,
            value,
            sequence,
            lock_time,
            sighash_type,
            other_outputs[0].clone(),
            other_outputs[1].clone(),
            other_outputs[2].clone(),
            other_outputs[3].clone(),
            other_outputs[4].clone(),
            other_outputs[5].clone(),
            if_flag,
            self.input.script.clone().into_bytes(),
        ])
    }
}

#[derive(Debug, Clone)]
pub struct Input {
    pub tx_in: TxIn,
    pub original_tx_out: TxOut,
    pub blinding_key: SecretKey,
}

impl Input {
    fn into_unblinded_input<C>(self, secp: &Secp256k1<C>) -> Result<UnblindedInput>
    where
        C: Verification,
    {
        let tx_in = self.tx_in;
        let confidential = self
            .original_tx_out
            .into_confidential()
            .with_context(|| format!("input {} is not confidential", tx_in.previous_output))?;

        let unblinded = confidential.unblind(secp, self.blinding_key)?;

        Ok(UnblindedInput {
            tx_in,
            confidential,
            unblinded,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UnblindedInput {
    pub tx_in: TxIn,
    pub confidential: ConfidentialTxOut,
    pub unblinded: UnblindedTxOut,
}

// TODO: Take rng param
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
