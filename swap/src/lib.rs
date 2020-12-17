use anyhow::{Context, Result};
use elements_fun::{
    bitcoin::{secp256k1::Message, Amount},
    bitcoin_hashes::{hash160, Hash},
    confidential::ValueCommitment,
    opcodes,
    script::Builder,
    sighash::SigHashCache,
    Address, AssetId, OutPoint, SigHashType, Transaction, TxIn, TxOut, UnblindedTxOut,
};
use secp256k1::{
    rand::{CryptoRng, RngCore},
    PublicKey as SecpPublicKey, Secp256k1, SecretKey, Verification, SECP256K1,
};
use serde::Deserialize;
use std::future::Future;

/// Sent from Alice to Bob, assuming Alice has bitcoin.
#[derive(Deserialize)]
pub struct Message0 {
    pub input: TxIn,
    pub input_as_txout: TxOut,
    pub input_blinding_sk: SecretKey,
    pub address_redeem: Address,
    pub address_change: Address,
    #[serde(with = "::elements_fun::bitcoin::util::amount::serde::as_sat")]
    pub fee: Amount,
}

/// Sent from Bob to Alice.
pub struct Message1 {
    pub transaction: Transaction,
}

pub struct Alice0 {
    redeem_amount_alice: Amount,
    redeem_amount_bob: Amount,
    input: TxIn,
    input_as_txout: TxOut,
    input_sk: SecretKey,
    input_blinding_sk: SecretKey,
    asset_id_bob: AssetId,
    address_redeem: Address,
    blinding_sk_redeem: SecretKey,
    address_change: Address,
    blinding_sk_change: SecretKey,
    fee: Amount,
}

impl Alice0 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        amount_alice: Amount,
        amount_bob: Amount,
        input: (OutPoint, TxOut),
        input_sk: SecretKey,
        input_blinding_sk: SecretKey,
        asset_id_bob: AssetId,
        address_redeem: Address,
        blinding_sk_redeem: SecretKey,
        address_change: Address,
        blinding_sk_change: SecretKey,
        fee: Amount,
    ) -> Self {
        let input_as_txout = input.1;
        let input = TxIn {
            previous_output: input.0,
            is_pegin: false,
            has_issuance: false,
            script_sig: Default::default(),
            sequence: 0xFFFF_FFFF,
            asset_issuance: Default::default(),
            witness: Default::default(),
        };

        Self {
            redeem_amount_alice: amount_alice,
            redeem_amount_bob: amount_bob,
            input,
            input_as_txout,
            input_sk,
            input_blinding_sk,
            asset_id_bob,
            address_redeem,
            blinding_sk_redeem,
            address_change,
            blinding_sk_change,
            fee,
        }
    }

    pub fn compose(&self) -> Message0 {
        Message0 {
            input: self.input.clone(),
            input_as_txout: self.input_as_txout.clone(),
            input_blinding_sk: self.input_blinding_sk,
            address_redeem: self.address_redeem.clone(),
            address_change: self.address_change.clone(),
            fee: self.fee,
        }
    }

    pub fn interpret(self, msg: Message1) -> Result<Transaction> {
        let secp = elements_fun::bitcoin::secp256k1::Secp256k1::new();

        let expected_redeem_asset_id_alice = self.asset_id_bob;
        let expected_redeem_amount_alice = self.redeem_amount_alice;
        msg.transaction
            .output
            .iter()
            .filter(|output| output.script_pubkey() == &self.address_redeem.script_pubkey())
            .map(|output| {
                let UnblindedTxOut {
                    asset: asset_id,
                    value: amount,
                    ..
                } = output
                    .as_confidential()
                    .context("not a confidential txout")?
                    .clone()
                    .unblind(&secp, self.blinding_sk_redeem)?;

                Result::<_>::Ok((asset_id, amount))
            })
            .find(|res| match res {
                Ok((asset_id, amount)) => {
                    asset_id == &expected_redeem_asset_id_alice
                        && amount == &expected_redeem_amount_alice.as_sat()
                }
                Err(_) => false,
            })
            .context("wrong redeem_output_alice")??;

        let input_as_confidential_txout = self
            .input_as_txout
            .as_confidential()
            .context("not a confidential txout")?;
        let UnblindedTxOut {
            asset: expected_change_asset_id_alice,
            value: input_amount_alice,
            ..
        } = input_as_confidential_txout
            .clone()
            .unblind(&secp, self.input_blinding_sk)?;
        let expected_change_amount_alice =
            Amount::from_sat(input_amount_alice) - self.redeem_amount_bob - self.fee;
        msg.transaction
            .output
            .iter()
            .filter(|output| output.script_pubkey() == &self.address_change.script_pubkey())
            .map(|output| {
                let UnblindedTxOut {
                    asset: asset_id,
                    value: amount,
                    ..
                } = output
                    .as_confidential()
                    .context("not a confidential txout")?
                    .clone()
                    .unblind(&secp, self.blinding_sk_change)?;

                Result::<_>::Ok((asset_id, amount))
            })
            .find(|res| match res {
                Ok((asset_id, amount)) => {
                    asset_id == &expected_change_asset_id_alice
                        && amount == &expected_change_amount_alice.as_sat()
                }
                Err(_) => false,
            })
            .context("wrong change_output_alice")??;

        // sign yourself and put signature in right spot
        let input_pk_alice = SecpPublicKey::from_secret_key(&secp, &self.input_sk);

        let mut transaction = msg.transaction;

        let input_index_alice = transaction
            .input
            .iter()
            .position(|input| input.previous_output == self.input.previous_output)
            .context("transaction does not contain input_alice")?;
        transaction.input[input_index_alice].witness.script_witness = {
            let hash = hash160::Hash::hash(&input_pk_alice.serialize());
            let script = Builder::new()
                .push_opcode(opcodes::all::OP_DUP)
                .push_opcode(opcodes::all::OP_HASH160)
                .push_slice(&hash.into_inner())
                .push_opcode(opcodes::all::OP_EQUALVERIFY)
                .push_opcode(opcodes::all::OP_CHECKSIG)
                .into_script();

            let hash = SigHashCache::new(&transaction).segwitv0_sighash(
                input_index_alice,
                &script,
                input_as_confidential_txout.value,
                SigHashType::All,
            );

            let sig = secp.sign(&Message::from(hash), &self.input_sk);

            let mut serialized_signature = sig.serialize_der().to_vec();
            serialized_signature.push(SigHashType::All as u8);

            vec![serialized_signature, input_pk_alice.serialize().to_vec()]
        };

        // publish transaction
        Ok(transaction)
    }
}

pub struct Bob0 {
    redeem_amount_alice: Amount,
    redeem_amount_bob: Amount,
    input: TxIn,
    input_as_txout: TxOut,
    input_blinding_sk: SecretKey,
    asset_id_alice: AssetId,
    address_redeem: Address,
    address_change: Address,
}

impl Bob0 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        redeem_amount_alice: Amount,
        redeem_amount_bob: Amount,
        input: (OutPoint, TxOut),
        input_blinding_sk: SecretKey,
        asset_id_alice: AssetId,
        address_redeem: Address,
        address_change: Address,
    ) -> Self {
        let input_as_txout = input.1;

        let input = TxIn {
            previous_output: input.0,
            is_pegin: false,
            has_issuance: false,
            script_sig: Default::default(),
            sequence: 0xFFFF_FFFF,
            asset_issuance: Default::default(),
            witness: Default::default(),
        };

        Self {
            redeem_amount_alice,
            redeem_amount_bob,
            input,
            input_as_txout,
            input_blinding_sk,
            asset_id_alice,
            address_redeem,
            address_change,
        }
    }

    pub fn interpret<R, C>(self, rng: &mut R, secp: &Secp256k1<C>, msg: Message0) -> Result<Bob1>
    where
        R: RngCore + CryptoRng,
        C: Verification,
    {
        let alice_input_as_txout = msg
            .input_as_txout
            .as_confidential()
            .context("not a confidential txout")?
            .clone();
        let bob_input_as_txout = self
            .input_as_txout
            .as_confidential()
            .context("not a confidential txout")?
            .clone();

        let UnblindedTxOut {
            asset: asset_id_alice,
            asset_blinding_factor: abf_in_alice,
            value_blinding_factor: vbf_in_alice,
            value: amount_in_alice,
        } = alice_input_as_txout.unblind(secp, msg.input_blinding_sk)?;
        let UnblindedTxOut {
            asset: asset_id_bob,
            asset_blinding_factor: abf_in_bob,
            value_blinding_factor: vbf_in_bob,
            value: amount_in_bob,
        } = bob_input_as_txout.unblind(secp, self.input_blinding_sk)?;

        let change_amount_alice = Amount::from_sat(amount_in_alice)
            .checked_sub(self.redeem_amount_bob)
            .map(|amount| amount.checked_sub(msg.fee))
            .flatten()
            .context("alice provided wrong amounts for the asset she's selling")?;
        let change_amount_bob = Amount::from_sat(amount_in_bob)
            .checked_sub(self.redeem_amount_alice)
            .context("alice provided wrong amounts for the asset she's buying")?;

        let input_alice = msg.input;
        let input_bob = self.input.clone();

        let inputs = [
            (
                asset_id_alice,
                amount_in_alice,
                alice_input_as_txout.asset,
                abf_in_alice,
                vbf_in_alice,
            ),
            (
                asset_id_bob,
                amount_in_bob,
                bob_input_as_txout.asset,
                abf_in_bob,
                vbf_in_bob,
            ),
        ];
        let (redeem_output_alice, abf_redeem_alice, vbf_redeem_alice) =
            TxOut::new_not_last_confidential(
                rng,
                &SECP256K1,
                self.redeem_amount_alice.as_sat(),
                msg.address_redeem,
                asset_id_bob,
                &inputs,
            )?;
        let (redeem_output_bob, abf_redeem_bob, vbf_redeem_bob) = TxOut::new_not_last_confidential(
            rng,
            &SECP256K1,
            self.redeem_amount_bob.as_sat(),
            self.address_redeem.clone(),
            self.asset_id_alice,
            &inputs,
        )?;
        let (change_output_alice, abf_change_alice, vbf_change_alice) =
            TxOut::new_not_last_confidential(
                rng,
                &SECP256K1,
                change_amount_alice.as_sat(),
                msg.address_change,
                self.asset_id_alice,
                &inputs,
            )?;

        let outputs = [
            (
                self.redeem_amount_alice.as_sat(),
                abf_redeem_alice,
                vbf_redeem_alice,
            ),
            (
                self.redeem_amount_bob.as_sat(),
                abf_redeem_bob,
                vbf_redeem_bob,
            ),
            (
                change_amount_alice.as_sat(),
                abf_change_alice,
                vbf_change_alice,
            ),
        ];

        let change_output_bob = TxOut::new_last_confidential(
            rng,
            &SECP256K1,
            change_amount_bob.as_sat(),
            self.address_change.clone(),
            asset_id_bob,
            &inputs,
            &outputs,
        )?;
        let fee = TxOut::new_fee(self.asset_id_alice, msg.fee.as_sat());

        let transaction = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![input_alice, input_bob],
            output: vec![
                redeem_output_alice,
                redeem_output_bob,
                change_output_alice,
                change_output_bob,
                fee,
            ],
        };

        let input_index_bob = transaction
            .input
            .iter()
            .position(|input| input.previous_output == self.input.previous_output)
            .context("transaction does not contain bob's input")?;

        Ok(Bob1 {
            transaction,
            input_index_bob,
            amount_in_bob: bob_input_as_txout.value,
        })
    }
}

pub struct Bob1 {
    transaction: Transaction,
    input_index_bob: usize,
    amount_in_bob: ValueCommitment,
}

impl Bob1 {
    pub async fn compose(
        &self,
        signer: impl Future<Output = Result<Transaction>>,
    ) -> Result<Message1> {
        let transaction = signer.await?;

        Ok(Message1 { transaction })
    }

    pub fn sign_with_key(&self, input_sk: &SecretKey) -> Result<Transaction> {
        let secp = elements_fun::bitcoin::secp256k1::Secp256k1::new();

        let input_pk_bob = SecpPublicKey::from_secret_key(&secp, &input_sk);

        let mut transaction = self.transaction.clone();
        transaction.input[self.input_index_bob]
            .witness
            .script_witness = {
            let hash = hash160::Hash::hash(&input_pk_bob.serialize());
            let script = Builder::new()
                .push_opcode(opcodes::all::OP_DUP)
                .push_opcode(opcodes::all::OP_HASH160)
                .push_slice(&hash.into_inner())
                .push_opcode(opcodes::all::OP_EQUALVERIFY)
                .push_opcode(opcodes::all::OP_CHECKSIG)
                .into_script();

            let sighash = SigHashCache::new(&transaction).segwitv0_sighash(
                self.input_index_bob,
                &script,
                self.amount_in_bob,
                SigHashType::All,
            );

            let sig = secp.sign(&Message::from(sighash), &input_sk);

            let mut serialized_signature = sig.serialize_der().to_vec();
            serialized_signature.push(SigHashType::All as u8);

            vec![serialized_signature, input_pk_bob.serialize().to_vec()]
        };

        Ok(transaction)
    }

    pub async fn sign_with_wallet<
        't,
        's: 't,
        S: FnOnce(&'t Transaction) -> F,
        F: Future<Output = Result<Transaction>> + 't,
    >(
        &'s self,
        signer: S,
    ) -> Result<Transaction> {
        signer(&self.transaction).await
    }
}
