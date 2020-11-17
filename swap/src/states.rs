use crate::make_txout;
use crate::unblind_asset_from_txout;
use anyhow::{anyhow, Result};
use bitcoin::Amount;
use bitcoin::Script;
use elements_fun::bitcoin::blockdata::opcodes;
use elements_fun::bitcoin::blockdata::script::Builder;
use elements_fun::bitcoin::secp256k1::Message;
use elements_fun::bitcoin::SigHashType;
use elements_fun::bitcoin_hashes::hash160;
use elements_fun::bitcoin_hashes::Hash;
use elements_fun::confidential::Asset;
use elements_fun::confidential::Nonce;
use elements_fun::confidential::Value;
use elements_fun::wally::{asset_final_vbf, tx_get_elements_signature_hash};
use elements_fun::Address;
use elements_fun::AssetId;
use elements_fun::OutPoint;
use elements_fun::Transaction;
use elements_fun::TxIn;
use elements_fun::TxOut;
use elements_fun::TxOutWitness;
use rand::CryptoRng;
use rand::RngCore;
use secp256k1::PublicKey;
use secp256k1::SecretKey;

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
            .filter(|output| output.script_pubkey == self.address_redeem.script_pubkey())
            .map(|output| {
                let (asset_id, _, _, _, amount) =
                    unblind_asset_from_txout(output.clone(), self.blinding_sk_redeem);

                (asset_id, amount)
            })
            .find(|(asset_id, amount)| {
                asset_id == &expected_redeem_asset_id_alice
                    && amount == &expected_redeem_amount_alice
            })
            .ok_or_else(|| anyhow!("wrong redeem_output_alice"))?;

        let (expected_change_asset_id_alice, _, _, _, input_amount_alice) =
            unblind_asset_from_txout(self.input_as_txout.clone(), self.input_blinding_sk);
        let expected_change_amount_alice = input_amount_alice - self.redeem_amount_bob - self.fee;
        msg.transaction
            .output
            .iter()
            .filter(|output| output.script_pubkey == self.address_change.script_pubkey())
            .map(|output| {
                let (asset_id, _, _, _, amount) =
                    unblind_asset_from_txout(output.clone(), self.blinding_sk_change);

                (asset_id, amount)
            })
            .find(|(asset_id, amount)| {
                asset_id == &expected_change_asset_id_alice
                    && amount == &expected_change_amount_alice
            })
            .ok_or_else(|| anyhow!("wrong change_output_alice"))?;

        // sign yourself and put signature in right spot
        let input_pk_alice = PublicKey::from_secret_key(&secp, &self.input_sk);
        let fund_amount_alice = self.input_as_txout.value;

        let mut transaction = msg.transaction;

        let input_index_alice = transaction
            .input
            .iter()
            .position(|input| input.previous_output == self.input.previous_output)
            .ok_or_else(|| anyhow!("transaction does not contain input_alice"))?;
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
        let (
            asset_id_alice,
            asset_id_commitment_in_alice,
            abf_in_alice,
            vbf_in_alice,
            amount_in_alice,
        ) = unblind_asset_from_txout(msg.input_as_txout.clone(), msg.input_blinding_sk);
        let (asset_id_bob, asset_id_commitment_in_bob, abf_in_bob, vbf_in_bob, amount_in_bob) =
            unblind_asset_from_txout(self.input_as_txout.clone(), self.input_blinding_sk);

        let abf_redeem_alice = SecretKey::new(rng);
        let abf_redeem_bob = SecretKey::new(rng);
        let abf_change_alice = SecretKey::new(rng);
        let abf_change_bob = SecretKey::new(rng);
        let abfs = vec![
            abf_in_alice.as_ref().to_vec(),
            abf_in_bob.as_ref().to_vec(),
            abf_redeem_alice.as_ref().to_vec(),
            abf_redeem_bob.as_ref().to_vec(),
            abf_change_alice.as_ref().to_vec(),
            abf_change_bob.as_ref().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let vbf_redeem_alice = SecretKey::new(rng);
        let vbf_redeem_bob = SecretKey::new(rng);
        let vbf_change_alice = SecretKey::new(rng);
        let vbfs = vec![
            vbf_in_alice.as_ref().to_vec(),
            vbf_in_bob.as_ref().to_vec(),
            vbf_redeem_alice.as_ref().to_vec(),
            vbf_redeem_bob.as_ref().to_vec(),
            vbf_change_alice.as_ref().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let change_amount_alice = amount_in_alice
            .checked_sub(self.redeem_amount_bob)
            .map(|amount| amount.checked_sub(msg.fee))
            .flatten()
            .ok_or_else(|| anyhow!("alice provided wrong amounts for the asset she's selling"))?;
        let change_amount_bob = amount_in_bob
            .checked_sub(self.redeem_amount_alice)
            .ok_or_else(|| anyhow!("alice provided wrong amounts for the asset she's buying"))?;

        let input_alice = msg.input;
        let input_bob = self.input.clone();

        let inputs = vec![
            (asset_id_alice, asset_id_commitment_in_alice, abf_in_alice),
            (asset_id_bob, asset_id_commitment_in_bob, abf_in_bob),
        ];

        let redeem_ephemeral_key_alice = SecretKey::new(rng);
        let redeem_output_alice = make_txout(
            rng,
            self.redeem_amount_alice,
            msg.address_redeem,
            asset_id_bob,
            *abf_redeem_alice.as_ref(),
            *vbf_redeem_alice.as_ref(),
            &inputs,
            redeem_ephemeral_key_alice,
        )?;

        let redeem_ephemeral_key_bob = SecretKey::new(rng);
        let redeem_output_bob = make_txout(
            rng,
            self.redeem_amount_bob,
            self.address_redeem.clone(),
            self.asset_id_alice,
            *abf_redeem_bob.as_ref(),
            *vbf_redeem_bob.as_ref(),
            &inputs,
            redeem_ephemeral_key_bob,
        )?;

        let change_ephemeral_key_alice = SecretKey::new(rng);
        let change_output_alice = make_txout(
            rng,
            change_amount_alice,
            msg.address_change,
            self.asset_id_alice,
            *abf_change_alice.as_ref(),
            *vbf_change_alice.as_ref(),
            &inputs,
            change_ephemeral_key_alice,
        )?;

        let vbf_change_bob = asset_final_vbf(
            vec![
                amount_in_alice.as_sat(),
                amount_in_bob.as_sat(),
                self.redeem_amount_alice.as_sat(),
                self.redeem_amount_bob.as_sat(),
                change_amount_alice.as_sat(),
                change_amount_bob.as_sat(),
            ],
            2,
            abfs,
            vbfs,
        );

        let change_ephemeral_key_bob = SecretKey::new(rng);
        let change_output_bob = make_txout(
            rng,
            change_amount_bob,
            self.address_change.clone(),
            asset_id_bob,
            *abf_change_bob.as_ref(),
            vbf_change_bob,
            &inputs,
            change_ephemeral_key_bob,
        )?;

        let fee = TxOut {
            asset: Asset::Explicit(self.asset_id_alice),
            value: Value::Explicit(msg.fee.as_sat()),
            nonce: Nonce::Null,
            script_pubkey: Script::default(),
            witness: TxOutWitness::default(),
        };

        let transaction = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![input_alice, input_bob.clone()],
            output: vec![
                redeem_output_alice.clone(),
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
            .ok_or_else(|| anyhow!("transaction does not contain bob's input"))?;

        Ok(Bob1 {
            transaction,
            input_index_bob,
            input_sk: self.input_sk,
            input_as_txout_bob: self.input_as_txout.clone(),
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
    pub fn compose(&self) -> Message1 {
        let secp = elements_fun::bitcoin::secp256k1::Secp256k1::new();

        let input_pk_bob = PublicKey::from_secret_key(&secp, &self.input_sk);
        let fund_bitcoin_tx_vout_bob = self.input_as_txout_bob.clone();
        let fund_amount_bob = fund_bitcoin_tx_vout_bob.value;

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

        Message1 { transaction }
    }
}

#[cfg(test)]
mod tests {
    use crate::make_confidential_address;
    use crate::states::{Alice0, Bob0};
    use anyhow::{anyhow, Result};
    use elements_fun::{Address, OutPoint, Transaction, TxOut};
    use elements_harness::elementd_rpc::ElementsRpc;
    use elements_harness::{Client, Elementsd};
    use rand::thread_rng;
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

        let asset_id_bob = client.issueasset(10.0, 0.0, true).await.unwrap().asset;
        let asset_id_alice = client.get_bitcoin_asset_id().await.unwrap();

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
            _final_sk_bob,
            _final_pk_bob,
            _final_blinding_sk_bob,
            _final_blinding_pk_bob,
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
            client.get_raw_transaction(fund_alice_txid).await.unwrap(),
            fund_address_alice,
        )
        .unwrap();

        let input_bob = extract_input(
            client.get_raw_transaction(fund_bob_txid).await.unwrap(),
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
            final_address_alice,
            final_blinding_sk_alice,
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
            final_address_bob,
        );

        let message0 = alice.compose();
        let bob1 = bob.interpret(&mut thread_rng(), message0).unwrap();
        let message1 = bob1.compose();
        let transaction = alice.interpret(message1).unwrap();
        let _txid = client.send_raw_transaction(&transaction).await.unwrap();
    }

    fn extract_input(tx: Transaction, address: Address) -> Result<(OutPoint, TxOut)> {
        let vout = tx
            .output
            .iter()
            .position(|output| output.script_pubkey == address.script_pubkey())
            .ok_or_else(|| anyhow!("Tx doesn't pay to address"))?;

        let outpoint = OutPoint {
            txid: tx.txid(),
            vout: vout as u32,
        };
        let tx_out = tx.output[vout].clone();
        Ok((outpoint, tx_out))
    }
}
