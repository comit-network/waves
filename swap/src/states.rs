use crate::make_txout;
use crate::unblind_asset_from_txout;
use anyhow::{bail, Result};
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
use elements_fun::encode::serialize_hex;
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
    pub input: TxIn,
    pub input_as_txout: TxOut,
    pub input_blinding_sk: SecretKey,
    pub address_redeem: Address,
    pub abf_redeem: SecretKey,
    pub vbf_redeem: SecretKey,
    pub address_change: Address,
    pub abf_change: SecretKey,
    pub vbf_change: SecretKey,
    pub fee: Amount,
}

/// Sent from Bob to Alice.
pub struct Message1 {
    pub input: TxIn,
    // Bob's input
    pub input_as_txout: TxOut,
    pub input_blinding_sk: SecretKey,

    pub asset_id_in: AssetId,
    pub asset_id_commitment_in: Asset,
    pub abf_in: SecretKey,
    pub address_redeem: Address,
    pub address_change: Address,
    pub abf_redeem: SecretKey,
    pub vbf_redeem: SecretKey,
    pub abf_change: SecretKey,
    pub witness_stack_bob: Vec<Vec<u8>>,
}

pub struct Alice0 {
    pub redeem_amount_alice: Amount,
    pub redeem_amount_bob: Amount,
    pub input: TxIn,
    pub input_as_txout: TxOut,
    pub input_sk: SecretKey,
    pub input_blinding_sk: SecretKey,
    pub asset_id_bob: AssetId,
    pub address_redeem: Address,
    pub abf_redeem: SecretKey,
    pub vbf_redeem: SecretKey,
    pub address_change: Address,
    pub abf_change: SecretKey,
    pub vbf_change: SecretKey,
    pub fee: Amount,
}

impl Alice0 {
    pub fn new<R>(
        rng: &mut R,
        amount_alice: Amount,
        amount_bob: Amount,
        // TODO: Define struct
        input: (OutPoint, TxOut),
        input_sk: SecretKey,
        input_blinding_sk: SecretKey,
        asset_id_bob: AssetId,
        address_redeem: Address,
        address_change: Address,
        fee: Amount,
    ) -> Self
    where
        R: RngCore + CryptoRng,
    {
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

        let abf_redeem = SecretKey::new(rng);
        let vbf_redeem = SecretKey::new(rng);

        let abf_change = SecretKey::new(rng);
        let vbf_change = SecretKey::new(rng);

        Self {
            redeem_amount_alice: amount_alice,
            redeem_amount_bob: amount_bob,
            input,
            input_as_txout,
            input_sk,
            input_blinding_sk,
            asset_id_bob,
            address_redeem,
            abf_redeem,
            vbf_redeem,
            address_change,
            abf_change,
            vbf_change,
            fee,
        }
    }

    pub fn compose(&self) -> Message0 {
        Message0 {
            input: self.input.clone(),
            input_as_txout: self.input_as_txout.clone(),
            input_blinding_sk: self.input_blinding_sk,
            address_redeem: self.address_redeem.clone(),
            abf_redeem: self.abf_redeem,
            vbf_redeem: self.vbf_redeem,
            address_change: self.address_change.clone(),
            abf_change: self.abf_change,
            vbf_change: self.vbf_change,
            fee: self.fee,
        }
    }

    pub fn interpret<R>(self, rng: &mut R, msg: Message1) -> Result<Transaction>
    where
        R: RngCore + CryptoRng,
    {
        let secp = elements_fun::bitcoin::secp256k1::Secp256k1::new();

        // todo verify that what received was expected

        let (asset_id_bob, asset_id_commitment_in_bob, abf_in_bob, vbf_in_bob, amount_in_bob) =
            unblind_asset_from_txout(msg.input_as_txout, msg.input_blinding_sk);

        if asset_id_bob != self.asset_id_bob {
            bail!(
                "Bob provided wrong asset: expected {}, got {}",
                self.asset_id_bob,
                asset_id_bob
            )
        }

        let (
            asset_id_alice,
            asset_id_commitment_in_alice,
            abf_in_alice,
            vbf_in_alice,
            amount_in_alice,
        ) = unblind_asset_from_txout(self.input_as_txout.clone(), self.input_blinding_sk);

        let abfs = vec![
            abf_in_alice.as_ref().to_vec(),
            abf_in_bob.as_ref().to_vec(),
            self.abf_redeem.as_ref().to_vec(),
            msg.abf_redeem.as_ref().to_vec(),
            self.abf_change.as_ref().to_vec(),
            msg.abf_change.as_ref().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let vbfs = vec![
            vbf_in_alice.as_ref().to_vec(),
            vbf_in_bob.as_ref().to_vec(),
            self.vbf_redeem.as_ref().to_vec(),
            msg.vbf_redeem.as_ref().to_vec(),
            self.vbf_change.as_ref().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let change_amount_alice = amount_in_alice - self.redeem_amount_bob - self.fee;
        let change_amount_bob = amount_in_bob - self.redeem_amount_alice;

        let input_alice = self.input.clone();
        let input_bob = msg.input;

        let inputs = vec![
            (asset_id_alice, asset_id_commitment_in_alice, abf_in_alice),
            (asset_id_bob, asset_id_commitment_in_bob, abf_in_bob),
        ];

        let redeem_output_alice = make_txout(
            rng,
            self.redeem_amount_alice,
            self.address_redeem,
            asset_id_bob,
            *self.abf_redeem.as_ref(),
            *self.vbf_redeem.as_ref(),
            &inputs,
        );

        let redeem_output_bob = make_txout(
            rng,
            self.redeem_amount_bob,
            msg.address_redeem.clone(),
            asset_id_alice,
            *msg.abf_redeem.as_ref(),
            *msg.vbf_redeem.as_ref(),
            &inputs,
        );

        let change_output_alice = make_txout(
            rng,
            change_amount_alice,
            self.address_change,
            asset_id_alice,
            *self.abf_change.as_ref(),
            *self.vbf_change.as_ref(),
            &inputs,
        );

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
        let change_output_bob = make_txout(
            rng,
            change_amount_bob,
            msg.address_change.clone(),
            asset_id_bob,
            *msg.abf_change.as_ref(),
            vbf_change_bob,
            &inputs,
        );

        let fee = TxOut {
            asset: Asset::Explicit(asset_id_alice),
            value: Value::Explicit(self.fee.as_sat()),
            nonce: Nonce::Null,
            script_pubkey: Script::default(),
            witness: TxOutWitness::default(),
        };

        let mut transaction = Transaction {
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

        dbg!(serialize_hex(&transaction));

        // extract signature from message and put it into the right spot
        // TODO: verify this is the correct position
        transaction.input[1].witness.script_witness = msg.witness_stack_bob;

        // sign yourself and put signature in right spot
        let input_pk_alice = PublicKey::from_secret_key(&secp, &self.input_sk);
        let fund_bitcoin_tx_vout_alice = self.input_as_txout.clone();
        let fund_amount_alice = fund_bitcoin_tx_vout_alice.value;

        // TODO: verify this is the correct position
        transaction.input[0].witness.script_witness = {
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
                0, // todo: ensure that this is Alice's input
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
    pub redeem_amount_alice: Amount,
    pub redeem_amount_bob: Amount,
    pub input: TxIn,
    pub input_as_txout: TxOut,
    pub input_sk: SecretKey,
    pub input_blinding_sk: SecretKey,
    pub asset_id_alice: AssetId,
    pub address_redeem: Address,
    pub abf_redeem: SecretKey,
    pub vbf_redeem: SecretKey,
    pub address_change: Address,
    pub abf_change: SecretKey,
    pub vbf_change: SecretKey,
}

impl Bob0 {
    pub fn new<R>(
        rng: &mut R,
        amount_alice: Amount,
        amount_bob: Amount,
        // TODO: Define struct
        input: (OutPoint, TxOut),
        input_sk: SecretKey,
        input_blinding_sk: SecretKey,
        asset_id_alice: AssetId,
        address_redeem: Address,
        address_change: Address,
    ) -> Self
    where
        R: RngCore + CryptoRng,
    {
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

        let abf_redeem = SecretKey::new(rng);
        let vbf_redeem = SecretKey::new(rng);

        let abf_change = SecretKey::new(rng);
        let vbf_change = SecretKey::new(rng);

        Self {
            redeem_amount_alice: amount_alice,
            redeem_amount_bob: amount_bob,
            input,
            input_as_txout,
            input_sk,
            input_blinding_sk,
            asset_id_alice,
            address_redeem,
            abf_redeem,
            vbf_redeem,
            address_change,
            abf_change,
            vbf_change,
        }
    }

    pub fn interpret<R>(self, rng: &mut R, msg: Message0) -> Bob1
    where
        R: RngCore + CryptoRng,
    {
        // TODO: Verify amounts and assets

        let (
            asset_id_alice,
            asset_id_commitment_in_alice,
            abf_in_alice,
            vbf_in_alice,
            amount_in_alice,
        ) = unblind_asset_from_txout(msg.input_as_txout, msg.input_blinding_sk);
        let (asset_id_bob, asset_id_commitment_in_bob, abf_in_bob, vbf_in_bob, amount_in_bob) =
            unblind_asset_from_txout(self.input_as_txout.clone(), self.input_blinding_sk);

        let abfs = vec![
            abf_in_alice.as_ref().to_vec(),
            abf_in_bob.as_ref().to_vec(),
            msg.abf_redeem.as_ref().to_vec(),
            self.abf_redeem.as_ref().to_vec(),
            msg.abf_change.as_ref().to_vec(),
            self.abf_change.as_ref().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let vbfs = vec![
            vbf_in_alice.as_ref().to_vec(),
            vbf_in_bob.as_ref().to_vec(),
            msg.vbf_redeem.as_ref().to_vec(),
            self.vbf_redeem.as_ref().to_vec(),
            msg.vbf_change.as_ref().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let change_amount_alice = amount_in_alice - self.redeem_amount_bob - msg.fee;
        let change_amount_bob = amount_in_bob - self.redeem_amount_alice;

        let input_alice = msg.input;
        let input_bob = self.input.clone();

        let inputs = vec![
            (asset_id_alice, asset_id_commitment_in_alice, abf_in_alice),
            (asset_id_bob, asset_id_commitment_in_bob, abf_in_bob),
        ];

        let redeem_output_alice = make_txout(
            rng,
            self.redeem_amount_alice,
            msg.address_redeem,
            asset_id_bob,
            *msg.abf_redeem.as_ref(),
            *msg.vbf_redeem.as_ref(),
            &inputs,
        );

        let redeem_output_bob = make_txout(
            rng,
            self.redeem_amount_bob,
            self.address_redeem.clone(),
            self.asset_id_alice,
            *self.abf_redeem.as_ref(),
            *self.vbf_redeem.as_ref(),
            &inputs,
        );

        let change_output_alice = make_txout(
            rng,
            change_amount_alice,
            msg.address_change,
            self.asset_id_alice,
            *msg.abf_change.as_ref(),
            *msg.vbf_change.as_ref(),
            &inputs,
        );

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
        let change_output_bob = make_txout(
            rng,
            change_amount_bob,
            self.address_change.clone(),
            asset_id_bob,
            *self.abf_change.as_ref(),
            vbf_change_bob,
            &inputs,
        );

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

        dbg!(serialize_hex(&transaction));

        Bob1 {
            transaction,
            input_bob,
            input_sk: self.input_sk,
            input_blinding_sk: self.input_blinding_sk,
            input_as_txout_bob: self.input_as_txout.clone(),
            asset_id_bob,
            asset_id_commitment_in_bob,
            abf_in_bob,
            address_redeem_bob: self.address_redeem.clone(),
            address_change_bob: self.address_change.clone(),
            abf_redeem: self.abf_redeem,
            vbf_redeem: self.vbf_redeem,
            abf_change: self.abf_change,
            vbf_change: self.vbf_change,
        }
    }
}

pub struct Bob1 {
    pub transaction: Transaction,
    pub input_bob: TxIn,
    pub input_sk: SecretKey,
    pub input_blinding_sk: SecretKey,
    pub input_as_txout_bob: TxOut,
    pub asset_id_bob: AssetId,
    pub asset_id_commitment_in_bob: Asset,
    pub abf_in_bob: SecretKey,
    pub address_redeem_bob: Address,
    pub address_change_bob: Address,
    pub abf_redeem: SecretKey,
    pub vbf_redeem: SecretKey,
    pub abf_change: SecretKey,
    pub vbf_change: SecretKey,
}

impl Bob1 {
    pub fn compose(&self) -> Message1 {
        let secp = elements_fun::bitcoin::secp256k1::Secp256k1::new();

        let input_pk_bob = PublicKey::from_secret_key(&secp, &self.input_sk);
        let fund_bitcoin_tx_vout_bob = self.input_as_txout_bob.clone();
        let fund_amount_bob = fund_bitcoin_tx_vout_bob.value;

        let witness_stack = {
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
                1, // todo: ensure that this is Bob;s input
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

        Message1 {
            input: self.input_bob.clone(),
            asset_id_in: self.asset_id_bob,
            asset_id_commitment_in: self.asset_id_commitment_in_bob,
            abf_in: self.abf_in_bob,
            address_redeem: self.address_redeem_bob.clone(),
            address_change: self.address_change_bob.clone(),
            abf_redeem: self.abf_redeem,
            vbf_redeem: self.vbf_redeem,
            abf_change: self.abf_change,
            witness_stack_bob: witness_stack,
            // bob's input information
            input_as_txout: self.input_as_txout_bob.clone(),
            input_blinding_sk: self.input_blinding_sk,
        }
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
            _final_blinding_sk_alice,
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
            &mut thread_rng(),
            amount_alice,
            amount_bob,
            input_alice,
            fund_sk_alice,
            fund_blinding_sk_alice,
            asset_id_bob,
            final_address_alice.clone(),
            final_address_alice,
            fee,
        );

        let bob = Bob0::new(
            &mut thread_rng(),
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
        let bob1 = bob.interpret(&mut thread_rng(), message0);
        let message1 = bob1.compose();
        let transaction = alice.interpret(&mut thread_rng(), message1).unwrap();
        client.send_raw_transaction(&transaction).await.unwrap();
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
