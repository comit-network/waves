use anyhow::{Context, Result};
use bitcoin::Amount;
use elements_fun::{
    bitcoin::{secp256k1::Message, SigHashType},
    bitcoin_hashes::{hash160, Hash},
    opcodes,
    script::Builder,
    wally::tx_get_elements_signature_hash,
    Address, AssetId, OutPoint, Transaction, TxIn, TxOut, UnblindedTxOut,
};
use rand::{CryptoRng, RngCore};
use secp256k1::{PublicKey as SecpPublicKey, SecretKey, SECP256K1};

/// Sent from Alice to Bob, assuming Alice has bitcoin.
pub struct Message0 {
    input: TxIn,
    input_as_txout: TxOut,
    input_blinding_sk: SecretKey,
    address_redeem: Address,
    address_change: Address,
    fee: Amount,
}

/// Sent from Bob to Alice.
pub struct Message1 {
    transaction: Transaction,
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
                    .unblind(self.blinding_sk_redeem)?;

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

        let UnblindedTxOut {
            asset: expected_change_asset_id_alice,
            value: input_amount_alice,
            ..
        } = self
            .input_as_txout
            .as_confidential()
            .context("not a confidential txout")?
            .clone()
            .unblind(self.input_blinding_sk)?;
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
                    .unblind(self.blinding_sk_change)?;

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
        let fund_amount_alice = self
            .input_as_txout
            .as_confidential()
            .context("not a confidential txout")?
            .value;

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

            let digest = tx_get_elements_signature_hash(
                &transaction,
                input_index_alice,
                &script,
                &fund_amount_alice,
                1,
                true,
            );

            let sig = secp.sign(&Message::from_slice(&digest.into_inner())?, &self.input_sk);

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
    input_sk: SecretKey,
    input_blinding_sk: SecretKey,
    asset_id_alice: AssetId,
    address_redeem: Address,
    address_change: Address,
}

impl Bob0 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        amount_alice: Amount,
        amount_bob: Amount,
        input: (OutPoint, TxOut),
        input_sk: SecretKey,
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
            redeem_amount_alice: amount_alice,
            redeem_amount_bob: amount_bob,
            input,
            input_as_txout,
            input_sk,
            input_blinding_sk,
            asset_id_alice,
            address_redeem,
            address_change,
        }
    }

    pub fn interpret<R>(self, rng: &mut R, msg: Message0) -> Result<Bob1>
    where
        R: RngCore + CryptoRng,
    {
        let alice_txout = msg
            .input_as_txout
            .as_confidential()
            .context("not a confidential txout")?
            .clone();
        let bob_txout = self
            .input_as_txout
            .as_confidential()
            .context("not a confidential txout")?
            .clone();

        let UnblindedTxOut {
            asset: asset_id_alice,
            asset_blinding_factor: abf_in_alice,
            value_blinding_factor: vbf_in_alice,
            value: amount_in_alice,
        } = alice_txout.unblind(msg.input_blinding_sk)?;
        let UnblindedTxOut {
            asset: asset_id_bob,
            asset_blinding_factor: abf_in_bob,
            value_blinding_factor: vbf_in_bob,
            value: amount_in_bob,
        } = bob_txout.unblind(self.input_blinding_sk)?;

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
                alice_txout.asset,
                abf_in_alice,
                vbf_in_alice,
            ),
            (
                asset_id_bob,
                amount_in_bob,
                bob_txout.asset,
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
            input_sk: self.input_sk,
            input_as_txout_bob: self.input_as_txout,
        })
    }
}

pub struct Bob1 {
    transaction: Transaction,
    input_index_bob: usize,
    input_sk: SecretKey,
    input_as_txout_bob: TxOut,
}

impl Bob1 {
    pub fn compose(&self) -> Result<Message1> {
        let secp = elements_fun::bitcoin::secp256k1::Secp256k1::new();

        let input_pk_bob = SecpPublicKey::from_secret_key(&secp, &self.input_sk);
        let fund_bitcoin_tx_vout_bob = self.input_as_txout_bob.clone();
        let fund_amount_bob = fund_bitcoin_tx_vout_bob
            .as_confidential()
            .context("not a confidential txout")?
            .value;

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

            let digest = tx_get_elements_signature_hash(
                &self.transaction,
                self.input_index_bob,
                &script,
                &fund_amount_bob,
                1,
                true,
            );

            let sig = secp.sign(
                &Message::from_slice(&digest.into_inner()).unwrap(),
                &self.input_sk,
            );

            let mut serialized_signature = sig.serialize_der().to_vec();
            serialized_signature.push(SigHashType::All as u8);

            vec![serialized_signature, input_pk_bob.serialize().to_vec()]
        };

        Ok(Message1 { transaction })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        make_confidential_address,
        states::{Alice0, Bob0},
    };
    use anyhow::Result;
    use elements_fun::{
        bitcoin::{Network, PrivateKey, PublicKey},
        encode::serialize_hex,
        Txid,
    };
    use elements_harness::{elementd_rpc::ElementsRpc, Client, Elementsd};
    use rand::thread_rng;
    use secp256k1::{Message, SecretKey, SECP256K1};
    use testcontainers::clients::Cli;

    #[tokio::test]
    async fn collaborative_create_and_sign() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let asset_id_alice = client.get_bitcoin_asset_id().await.unwrap();
        let asset_id_bob = client.issueasset(10.0, 0.0, true).await.unwrap().asset;

        // fund keypairs and addresses
        let (
            fund_address_alice,
            fund_sk_alice,
            _fund_pk_alice,
            fund_blinding_sk_alice,
            _fund_blinding_pk_alice,
        ) = make_confidential_address();
        let (
            fund_address_bob,
            fund_sk_bob,
            _fund_pk_bob,
            fund_blinding_sk_bob,
            _fund_blinding_pk_bob,
        ) = make_confidential_address();

        // redeem keypairs and addresses
        let (
            final_address_alice,
            _final_sk_alice,
            _final_pk_alice,
            final_blinding_sk_alice,
            _final_blinding_pk_alice,
        ) = make_confidential_address();
        let (
            final_address_bob,
            final_sk_bob,
            _final_pk_bob,
            final_blinding_sk_bob,
            _final_blinding_pk_bob,
        ) = make_confidential_address();

        // change keypairs and addresses
        let (
            change_address_alice,
            change_sk_alice,
            _change_pk_alice,
            change_blinding_sk_alice,
            _change_blinding_pk_alice,
        ) = make_confidential_address();
        let (
            change_address_bob,
            _change_sk_bob,
            _change_pk_bob,
            _change_blinding_sk_bob,
            _change_blinding_pk_bob,
        ) = make_confidential_address();

        // initial funding
        let fund_amount_alice = bitcoin::Amount::ONE_BTC;
        let fund_amount_bob = bitcoin::Amount::ONE_BTC;

        let fund_alice_txid = client
            .send_asset_to_address(
                fund_address_alice.clone(),
                fund_amount_alice,
                Some(asset_id_alice),
            )
            .await
            .unwrap();

        let fund_bob_txid = client
            .send_asset_to_address(
                fund_address_bob.clone(),
                fund_amount_bob,
                Some(asset_id_bob),
            )
            .await
            .unwrap();

        let amount_alice = bitcoin::Amount::from_sat(50_000_000);
        let amount_bob = bitcoin::Amount::from_sat(25_000_000);
        let fee = bitcoin::Amount::from_sat(900_000);

        let input_alice = extract_input(
            &client.get_raw_transaction(fund_alice_txid).await.unwrap(),
            fund_address_alice,
        )
        .unwrap();

        let input_bob = extract_input(
            &client.get_raw_transaction(fund_bob_txid).await.unwrap(),
            fund_address_bob,
        )
        .unwrap();

        let alice = Alice0::new(
            amount_alice,
            amount_bob,
            input_alice,
            fund_sk_alice,
            fund_blinding_sk_alice,
            asset_id_bob,
            final_address_alice.clone(),
            final_blinding_sk_alice,
            change_address_alice.clone(),
            change_blinding_sk_alice,
            fee,
        );

        let bob = Bob0::new(
            amount_alice,
            amount_bob,
            input_bob,
            fund_sk_bob,
            fund_blinding_sk_bob,
            asset_id_alice,
            final_address_bob.clone(),
            change_address_bob.clone(),
        );

        let message0 = alice.compose();
        let bob1 = bob.interpret(&mut thread_rng(), message0).unwrap();
        let message1 = bob1.compose().unwrap();

        let tx = alice.interpret(message1).unwrap();
        let _txid = client.send_raw_transaction(&tx).await.unwrap();

        let (final_output_bob, _) = extract_input(&tx, final_address_bob).unwrap();
        let _txid = move_output_to_wallet(
            &client,
            final_output_bob,
            final_sk_bob,
            final_blinding_sk_bob,
        )
        .await
        .unwrap();

        let (change_output_alice, _) = extract_input(&tx, change_address_alice).unwrap();
        let _txid = move_output_to_wallet(
            &client,
            change_output_alice,
            change_sk_alice,
            change_blinding_sk_alice,
        )
        .await
        .unwrap();
    }

    // TODO: Only works with Bitcoin. Support other assets
    async fn move_output_to_wallet(
        client: &Client,
        previous_output: OutPoint,
        previous_output_sk: SecretKey,
        previous_output_blinding_sk: SecretKey,
    ) -> Result<Txid> {
        #[allow(clippy::cast_possible_truncation)]
        let input = TxIn {
            previous_output,
            is_pegin: false,
            has_issuance: false,
            script_sig: Default::default(),
            sequence: 0xFFFF_FFFF,
            asset_issuance: Default::default(),
            witness: Default::default(),
        };

        let previous_output_tx = client.get_raw_transaction(previous_output.txid).await?;
        let previous_output = previous_output_tx.output[previous_output.vout as usize].clone();

        let txout = previous_output
            .as_confidential()
            .context("not a confidential txout")?
            .clone();

        let UnblindedTxOut {
            asset: asset_id,
            asset_blinding_factor: abf_in,
            value_blinding_factor: vbf_in,
            value: amount_in,
        } = txout.unblind(previous_output_blinding_sk)?;

        let fee = 900_000;
        let amount_out = Amount::from_sat(amount_in - fee);

        let move_address = client.getnewaddress().await?;

        let inputs = [(asset_id, amount_in, txout.asset, abf_in, vbf_in)];

        let output = TxOut::new_last_confidential(
            &mut thread_rng(),
            &SECP256K1,
            amount_out.as_sat(),
            move_address,
            asset_id,
            &inputs,
            &[],
        )?;

        let fee = TxOut::new_fee(asset_id, fee);

        let mut tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![input],
            output: vec![output.clone(), fee],
        };

        let previous_output_pk = PublicKey::from_private_key(
            &SECP256K1,
            &PrivateKey {
                compressed: true,
                network: Network::Regtest,
                key: previous_output_sk,
            },
        );

        tx.input[0].witness.script_witness = {
            let hash = hash160::Hash::hash(&previous_output_pk.to_bytes());
            let script = Builder::new()
                .push_opcode(opcodes::all::OP_DUP)
                .push_opcode(opcodes::all::OP_HASH160)
                .push_slice(&hash.into_inner())
                .push_opcode(opcodes::all::OP_EQUALVERIFY)
                .push_opcode(opcodes::all::OP_CHECKSIG)
                .into_script();

            let digest = tx_get_elements_signature_hash(
                &tx,
                0,
                &script,
                &previous_output
                    .as_confidential()
                    .context("not a confidential txout")?
                    .value,
                1,
                true,
            );

            let sig = SECP256K1.sign(
                &Message::from_slice(&digest.into_inner())?,
                &previous_output_sk,
            );

            let mut serialized_signature = sig.serialize_der().to_vec();
            serialized_signature.push(SigHashType::All as u8);

            vec![serialized_signature, previous_output_pk.to_bytes()]
        };

        let tx_hex = serialize_hex(&tx);
        let txid = client.sendrawtransaction(tx_hex).await?;

        Ok(txid)
    }

    fn extract_input(tx: &Transaction, address: Address) -> Result<(OutPoint, TxOut)> {
        let vout = tx
            .output
            .iter()
            .position(|output| output.script_pubkey() == &address.script_pubkey())
            .context("Tx doesn't pay to address")?;

        let outpoint = OutPoint {
            txid: tx.txid(),
            vout: vout as u32,
        };
        let tx_out = tx.output[vout].clone();
        Ok((outpoint, tx_out))
    }
}
