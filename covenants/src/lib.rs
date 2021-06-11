use crate::stack_simulator::simulate;
use anyhow::{anyhow, bail, Context, Result};
use elements::{
    bitcoin::{util::psbt::serialize::Serialize, Amount, Network, PrivateKey, PublicKey},
    confidential::{Asset, AssetBlindingFactor, Value, ValueBlindingFactor},
    encode::Encodable,
    hashes::{sha256d, Hash},
    opcodes::all::*,
    script::Builder,
    secp256k1_zkp::{
        rand::{CryptoRng, RngCore},
        Secp256k1, SecretKey, Signature, Signing, Verification, SECP256K1,
    },
    sighash::SigHashCache,
    Address, AddressParams, AssetId, AssetIssuance, OutPoint, Script, SigHashType, Transaction,
    TxIn, TxInWitness, TxOut, TxOutSecrets,
};
use estimate_transaction_size::estimate_virtual_size;
use input::Input;
use secp256k1_zkp::{rand::thread_rng, SurjectionProof, Tag};
use std::future::Future;

#[cfg(test)]
mod protocol_tests;
mod stack_simulator;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoanRequest {
    #[serde(with = "::elements::bitcoin::util::amount::serde::as_sat")]
    pub collateral_amount: Amount,
    collateral_inputs: Vec<Input>,
    #[serde(with = "::elements::bitcoin::util::amount::serde::as_sat")]
    fee_sats_per_vbyte: Amount,
    borrower_pk: PublicKey,
    timelock: u64,
    borrower_address: Address,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoanResponse {
    transaction: Transaction,
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
    pub fn new<R>(
        rng: &mut R,
        address: Address,
        address_blinding_sk: SecretKey,
        collateral_amount: Amount,
        collateral_inputs: Vec<Input>,
        fee_sats_per_vbyte: Amount,
        timelock: u64,
        bitcoin_asset_id: AssetId,
        usdt_asset_id: AssetId,
    ) -> Result<Self>
    where
        R: RngCore + CryptoRng,
    {
        let keypair = make_keypair(rng);

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

    /// Interpret loan response from lender.
    ///
    /// This method does not check if the borrower agrees with the
    /// "repayment condition" i.e. the values in
    /// `repayment_collateral_input`. This belongs in a higher level,
    /// much like verifying that other loan conditions haven't
    /// changed.

    pub fn interpret<C>(self, secp: &Secp256k1<C>, loan_response: LoanResponse) -> Result<Borrower1>
    where
        C: Signing + Verification,
    {
        let transaction = loan_response.transaction;

        let principal_tx_out_amount = transaction
            .output
            .iter()
            .find_map(|out| {
                let unblinded_out = out.unblind(secp, self.address_blinding_sk).ok()?;
                let is_principal_out = unblinded_out.asset == self.usdt_asset_id
                    && out.script_pubkey == self.address.script_pubkey();

                is_principal_out.then(|| Amount::from_sat(unblinded_out.value))
            })
            .context("no principal txout")?;

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
            .find_map(|out| {
                let unblinded_out = out.unblind(secp, collateral_blinding_sk).ok()?;
                let is_collateral_out = unblinded_out.asset == self.bitcoin_asset_id
                    && unblinded_out.value == self.collateral_amount.as_sat()
                    && out.script_pubkey == collateral_script_pubkey;

                is_collateral_out.then(|| out)
            })
            .context("no collateral txout")?;

        let collateral_input_amount = self
            .collateral_inputs
            .iter()
            .map(|input| input.clone().into_unblinded_input(secp))
            .try_fold(0, |sum, input| {
                input.map(|input| sum + input.secrets.value).ok()
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
            .find_map(|out| {
                let unblinded_out = out.unblind(secp, self.address_blinding_sk).ok()?;
                let is_collateral_change_out = unblinded_out.asset == self.bitcoin_asset_id
                    && unblinded_out.value == collateral_change_amount.as_sat()
                    && out.script_pubkey == self.address.script_pubkey();

                is_collateral_change_out.then(|| out)
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
    // TODO: This name sucks. Thanks, Lucas.
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
        fee_sats_per_vbyte: Amount,
    ) -> Result<Transaction>
    where
        R: RngCore + CryptoRng,
        C: Verification + Signing,
        CS: FnOnce(Amount, AssetId) -> CF,
        CF: Future<Output = Result<Vec<Input>>>,
        SI: FnOnce(Transaction) -> SF,
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
                .map(|input| (input.txout.asset, &input.secrets))
                .collect::<Vec<_>>();
            borrower_inputs.push((collateral_input.txout.asset, &collateral_input.secrets));

            borrower_inputs
        };

        let mut repayment_principal_output = self.repayment_principal_output.clone();
        let domain = inputs
            .iter()
            .map(|(asset, secrets)| {
                Ok((
                    asset
                        .into_asset_gen(secp)
                        .ok_or_else(|| anyhow!("unexpected explicit or null asset"))?,
                    Tag::from(secrets.asset.into_inner().0),
                    secrets.asset_bf.into_inner(),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        repayment_principal_output.witness.surjection_proof = Some(SurjectionProof::new(
            secp,
            rng,
            Tag::from(self.usdt_asset_id.into_inner().0),
            // TODO: Consider changing upstream API to take Tweak
            // SecretKey::from_slice(&self.repayment_collateral_abf.into_inner()[..]).unwrap(),
            self.repayment_collateral_abf.into_inner(),
            domain.as_slice(),
        )?);

        let principal_input_amount = unblinded_principal_inputs
            .iter()
            .fold(0, |acc, input| acc + input.secrets.value);
        let change_amount = Amount::from_sat(principal_input_amount)
            .checked_sub(repayment_amount)
            .with_context(|| {
                format!(
                    "cannot pay for output {} with input {}",
                    repayment_amount, principal_input_amount,
                )
            })?;

        let principal_repayment_output = TxOutSecrets::new(
            self.usdt_asset_id,
            self.repayment_collateral_abf,
            repayment_amount.as_sat(),
            self.repayment_collateral_vbf,
        );
        let mut outputs = vec![principal_repayment_output];

        let mut tx_ins: Vec<TxIn> = unblinded_principal_inputs
            .clone()
            .into_iter()
            .map(|input| input.txin)
            .collect();
        tx_ins.push(collateral_input.txin);

        let inputs_not_last_confidential = inputs
            .iter()
            .copied()
            .map(|(asset, secrets)| (asset, Some(secrets)))
            .collect::<Vec<_>>();
        let change_output = match change_amount {
            Amount::ZERO => None,
            _ => {
                let (output, abf, vbf) = TxOut::new_not_last_confidential(
                    rng,
                    secp,
                    change_amount.as_sat(),
                    self.address.clone(),
                    self.usdt_asset_id,
                    &inputs_not_last_confidential,
                )
                .context("Change output creation failed")?;

                let principal_change_output =
                    TxOutSecrets::new(self.usdt_asset_id, abf, change_amount.as_sat(), vbf);
                outputs.push(principal_change_output);

                Some(output)
            }
        };
        let tx_fee = Amount::from_sat(
            estimate_virtual_size(tx_ins.len() as u64, 4) * fee_sats_per_vbyte.as_sat(),
        );
        let (collateral_output, _, _) = TxOut::new_last_confidential(
            rng,
            secp,
            (self.collateral_amount - tx_fee).as_sat(),
            self.address.clone(),
            self.bitcoin_asset_id,
            &inputs,
            outputs.iter().collect::<Vec<_>>().as_ref(),
        )
        .context("Creation of collateral output failed")?;

        let tx_fee_output = TxOut::new_fee(tx_fee.as_sat(), self.bitcoin_asset_id);

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
                self.repayment_collateral_input.original_txout.value,
                SigHashType::All,
            );

            let sig = SECP256K1.sign(
                &elements::secp256k1_zkp::Message::from(sighash),
                &self.keypair.0,
            );

            let script_witness = RepaymentWitnessStack::new(
                sig,
                self.keypair.1,
                self.repayment_collateral_input.original_txout.value,
                &tx,
                self.collateral_script.clone(),
            )
            .unwrap()
            .serialise()
            .unwrap();

            simulate(self.collateral_script.clone(), script_witness.clone()).unwrap();

            tx.input[1].witness = TxInWitness {
                amount_rangeproof: None,
                inflation_keys_rangeproof: None,
                script_witness,
                pegin_witness: vec![],
            };
        };

        // sign repayment input of the principal amount
        let tx = { signer(tx).await? };

        Ok(tx)
    }
}

pub struct Lender0 {
    keypair: (SecretKey, PublicKey),
    address: Address,
    bitcoin_asset_id: AssetId,
    usdt_asset_id: AssetId,
}

impl Lender0 {
    pub fn new<R>(
        rng: &mut R,
        bitcoin_asset_id: AssetId,
        usdt_asset_id: AssetId,
        address: Address,
    ) -> Result<Self>
    where
        R: RngCore + CryptoRng,
    {
        let keypair = make_keypair(rng);

        Ok(Self {
            bitcoin_asset_id,
            keypair,
            address,
            usdt_asset_id,
        })
    }

    pub async fn interpret<R, C, CS, CF>(
        self,
        rng: &mut R,
        secp: &Secp256k1<C>,
        coin_selector: CS,
        loan_request: LoanRequest,
    ) -> Result<Lender1>
    where
        R: RngCore + CryptoRng,
        C: Verification + Signing,
        CS: FnOnce(Amount, AssetId) -> CF,
        CF: Future<Output = Result<Vec<Input>>>,
    {
        let principal_amount = Lender0::calc_principal_amount(&loan_request);
        let collateral_inputs = loan_request
            .collateral_inputs
            .into_iter()
            .map(|input| input.into_unblinded_input(secp))
            .collect::<Result<Vec<_>>>()?;

        let borrower_inputs = collateral_inputs
            .iter()
            .map(|input| (input.txout.asset, &input.secrets));

        let principal_inputs = coin_selector(principal_amount, self.usdt_asset_id).await?;
        let unblinded_principal_inputs = principal_inputs
            .clone()
            .into_iter()
            .map(|input| input.into_unblinded_input(secp))
            .collect::<Result<Vec<_>>>()?;
        let lender_inputs = unblinded_principal_inputs
            .iter()
            .map(|input| (input.txout.asset, &input.secrets))
            .collect::<Vec<_>>();

        let inputs = borrower_inputs.chain(lender_inputs).collect::<Vec<_>>();

        let collateral_input_amount = collateral_inputs
            .iter()
            .fold(0, |sum, input| sum + input.secrets.value);

        let collateral_amount = loan_request.collateral_amount;

        let (repayment_principal_output, repayment_collateral_abf, repayment_collateral_vbf) = {
            let dummy_asset_id = self.usdt_asset_id;
            let dummy_abf = AssetBlindingFactor::new(rng);
            let dummy_asset = Asset::new_confidential(secp, dummy_asset_id, dummy_abf);
            let dummy_amount = principal_amount.as_sat();
            let dummy_vbf = ValueBlindingFactor::new(rng);
            let dummy_secrets =
                TxOutSecrets::new(dummy_asset_id, dummy_abf, dummy_amount, dummy_vbf);
            let dummy_inputs = [(dummy_asset, Some(&dummy_secrets))];

            TxOut::new_not_last_confidential(
                rng,
                secp,
                principal_amount.as_sat(),
                self.address.clone(),
                self.usdt_asset_id,
                &dummy_inputs,
            )?
        };

        let (_, lender_pk) = self.keypair;
        let collateral_script = loan_contract(
            loan_request.borrower_pk,
            lender_pk,
            loan_request.timelock,
            repayment_principal_output.clone(),
        )?;

        let (collateral_blinding_sk, collateral_blinding_pk) = make_keypair(&mut thread_rng());
        let collateral_address = Address::p2wsh(
            &collateral_script,
            Some(collateral_blinding_pk.key),
            &AddressParams::ELEMENTS,
        );
        let inputs_not_last_confidential = inputs
            .iter()
            .map(|(asset, secrets)| (*asset, Some(*secrets)))
            .collect::<Vec<_>>();
        let (collateral_tx_out, abf_collateral, vbf_collateral) = TxOut::new_not_last_confidential(
            rng,
            secp,
            collateral_amount.as_sat(),
            collateral_address.clone(),
            self.bitcoin_asset_id,
            inputs_not_last_confidential.as_slice(),
        )
        .context("could not construct collateral txout")?;

        let (principal_tx_out, abf_principal, vbf_principal) = TxOut::new_not_last_confidential(
            rng,
            secp,
            principal_amount.as_sat(),
            loan_request.borrower_address.clone(),
            self.usdt_asset_id,
            inputs_not_last_confidential.as_slice(),
        )
        .context("could not construct principal txout")?;

        let principal_input_amount = unblinded_principal_inputs
            .iter()
            .fold(0, |sum, input| sum + input.secrets.value);
        let principal_change_amount = Amount::from_sat(principal_input_amount) - principal_amount;
        let (principal_change_tx_out, abf_principal_change, vbf_principal_change) =
            TxOut::new_not_last_confidential(
                rng,
                secp,
                principal_change_amount.as_sat(),
                self.address.clone(),
                self.usdt_asset_id,
                &inputs_not_last_confidential,
            )
            .context("could not construct principal change txout")?;

        let not_last_confidential_outputs = [
            &TxOutSecrets::new(
                self.bitcoin_asset_id,
                abf_collateral,
                collateral_amount.as_sat(),
                vbf_collateral,
            ),
            &TxOutSecrets::new(
                self.usdt_asset_id,
                abf_principal,
                principal_amount.as_sat(),
                vbf_principal,
            ),
            &TxOutSecrets::new(
                self.usdt_asset_id,
                abf_principal_change,
                principal_change_amount.as_sat(),
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
        let (collateral_change_tx_out, _, _) = TxOut::new_last_confidential(
            rng,
            secp,
            collateral_change_amount.as_sat(),
            loan_request.borrower_address,
            self.bitcoin_asset_id,
            inputs
                .iter()
                .map(|(asset, secrets)| (*asset, *secrets))
                .collect::<Vec<_>>()
                .as_slice(),
            &not_last_confidential_outputs,
        )
        .context("Creation of collateral change output failed")?;

        let tx_ins = {
            let borrower_inputs = collateral_inputs.iter().map(|input| input.txin.clone());
            let lender_inputs = principal_inputs.iter().map(|input| input.txin.clone());
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

        let repayment_collateral_input = {
            let vout = loan_transaction
                .output
                .iter()
                .position(|out| out.script_pubkey == collateral_address.script_pubkey())
                .expect("loan transaction contains collateral output");

            Input {
                txin: TxIn {
                    previous_output: OutPoint {
                        txid: loan_transaction.txid(),
                        vout: vout as u32,
                    },
                    is_pegin: false,
                    has_issuance: false,
                    script_sig: Script::new(),
                    sequence: 0,
                    asset_issuance: AssetIssuance::default(),
                    witness: TxInWitness::default(),
                },
                original_txout: collateral_tx_out,
                blinding_key: collateral_blinding_sk,
            }
        };

        Ok(Lender1 {
            keypair: self.keypair,
            address: self.address,
            timelock: loan_request.timelock,
            loan_transaction,
            collateral_script,
            collateral_amount: loan_request.collateral_amount,
            repayment_collateral_input,
            repayment_collateral_abf,
            repayment_collateral_vbf,
            bitcoin_asset_id: self.bitcoin_asset_id,
            repayment_principal_output,
        })
    }

    // TODO: add some better logic here, or at least make it possible
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

        let inputs = [(collateral_input.txout.asset, &collateral_input.secrets)];

        let (collateral_output, _, _) = TxOut::new_last_confidential(
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

        let tx_ins = vec![collateral_input.txin];
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
            self.repayment_collateral_input.original_txout.value,
            SigHashType::All,
        );

        let sig = SECP256K1.sign(
            &elements::secp256k1_zkp::Message::from(sighash),
            &self.keypair.0,
        );

        let mut sig = sig.serialize_der().to_vec();
        sig.push(SigHashType::All as u8);
        let if_flag = vec![];

        liquidation_transaction.input[0].witness = TxInWitness {
            amount_rangeproof: None,
            inflation_keys_rangeproof: None,
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

        // input-specific values
        let (previous_out, script_0, script_1, script_2, value, sequence) = {
            let InputData {
                previous_output,
                script,
                value,
                sequence,
            } = &self.input;

            // a witness stack element cannot be larger than 80 bytes,
            // so we split the script into 3 to allow for a 240-byte
            // long script
            if script.len() > 240 {
                bail!("script larger than max size of 240 bytes");
            }

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

                // a witness stack element cannot be larger than 80
                // bytes, so we split each output into 2 to allow for
                // 160-byte long txouts
                if output.len() > 160 {
                    bail!("txout larger than max size of 160 bytes");
                }

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

pub fn make_keypair<R>(rng: &mut R) -> (SecretKey, PublicKey)
where
    R: RngCore + CryptoRng,
{
    let sk = SecretKey::new(rng);
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
