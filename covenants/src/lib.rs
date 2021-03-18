#[cfg(test)]
mod tests {
    use anyhow::Result;
    use elements::bitcoin::util::psbt::serialize::Serialize;
    use elements::bitcoin::{Amount, Network, PrivateKey, PublicKey};
    use elements::confidential::{Asset, Value};
    use elements::encode::Encodable;
    use elements::opcodes::all::*;
    use elements::script::Builder;
    use elements::secp256k1::rand::thread_rng;
    use elements::secp256k1::{SecretKey, Signature, SECP256K1};
    use elements::sighash::SigHashCache;
    use elements::{
        bitcoin::hashes::{sha256d, Hash},
        confidential,
    };
    use elements::{
        Address, AddressParams, OutPoint, Script, SigHashType, Transaction, TxIn, TxInWitness,
        TxOut,
    };
    use elements_harness::Client;
    use elements_harness::Elementsd;
    use testcontainers::clients::Cli;

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

    #[tokio::test]
    async fn it_works() {
        // start elements
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };
        let asset_id_lbtc = client.get_bitcoin_asset_id().await.unwrap();

        // create covenants script
        let (sk, pk) = make_keypair();
        let script = Builder::new()
            .push_opcode(OP_2SWAP)
            .push_opcode(OP_CAT)
            .push_opcode(OP_HASH256)
            .push_opcode(OP_ROT)
            .push_opcode(OP_ROT)
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
            .into_script();
        let address = Address::p2wsh(&script, None, &AddressParams::ELEMENTS);

        // fund covenants address
        let funding_amount = 100_000_000;
        let funding_value = Amount::from_sat(funding_amount);
        let txid = client
            .send_asset_to_address(&address, funding_value, None)
            .await
            .unwrap();

        let tx = client.get_raw_transaction(txid).await.unwrap();
        let vout = tx
            .output
            .iter()
            .position(|o| o.script_pubkey == address.script_pubkey())
            .unwrap() as u32;

        // spend
        let fee = 100_000;
        let address = Address::p2wpkh(&pk, None, &AddressParams::ELEMENTS);
        let mut tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![TxIn {
                previous_output: OutPoint { txid, vout },
                is_pegin: false,
                has_issuance: false,
                script_sig: Default::default(),
                sequence: 0,
                asset_issuance: Default::default(),
                witness: Default::default(),
            }],
            output: vec![
                TxOut {
                    asset: Asset::Explicit(asset_id_lbtc),
                    value: Value::Explicit(Amount::from_sat(funding_amount - fee).as_sat()),
                    nonce: Default::default(),
                    script_pubkey: address.script_pubkey(),
                    witness: Default::default(),
                },
                TxOut {
                    asset: Asset::Explicit(asset_id_lbtc),
                    value: Value::Explicit(Amount::from_sat(fee).as_sat()),
                    nonce: Default::default(),
                    script_pubkey: Default::default(),
                    witness: Default::default(),
                },
            ],
        };

        let sighash = SigHashCache::new(&tx).segwitv0_sighash(
            0,
            &script.clone(),
            Value::Explicit(Amount::from_sat(funding_amount).as_sat()),
            SigHashType::All,
        );

        let sig = SECP256K1.sign(&elements::secp256k1::Message::from(sighash), &sk);

        tx.input[0].witness = TxInWitness {
            amount_rangeproof: vec![],
            inflation_keys_rangeproof: vec![],
            script_witness: WitnessStack::new(sig, pk, funding_amount, &tx, script)
                .unwrap()
                .serialise()
                .unwrap(),
            pegin_witness: vec![],
        };

        client.send_raw_transaction(&tx).await.unwrap();
    }

    // Only supports 1 input and 2 outputs
    struct WitnessStack {
        sig: Signature,
        pk: PublicKey,
        tx_version: u32,
        hash_prev_out: elements::hashes::sha256d::Hash,
        hash_sequence: elements::hashes::sha256d::Hash,
        hash_issuances: elements::hashes::sha256d::Hash,
        input: InputData,
        principal_repayment_output: TxOut,
        tx_fee_output: TxOut,
        lock_time: u32,
        sighash_type: SigHashType,
    }

    struct InputData {
        previous_output: OutPoint,
        script: Script,
        value: confidential::Value,
        sequence: u32,
    }

    impl WitnessStack {
        fn new(
            sig: Signature,
            pk: PublicKey,
            funding_amount: u64,
            tx: &Transaction,
            script: Script,
        ) -> Result<Self> {
            let tx_version = tx.version;

            let hash_prev_out = {
                let mut enc = sha256d::Hash::engine();
                tx.input[0].previous_output.consensus_encode(&mut enc)?;
                sha256d::Hash::from_engine(enc)
            };

            let hash_sequence = {
                let mut enc = sha256d::Hash::engine();
                tx.input[0].sequence.consensus_encode(&mut enc)?;
                sha256d::Hash::from_engine(enc)
            };

            let hash_issuances = {
                let mut enc = sha256d::Hash::engine();
                if tx.input[0].has_issuance() {
                    tx.input[0].asset_issuance.consensus_encode(&mut enc)?;
                } else {
                    0u8.consensus_encode(&mut enc)?;
                }
                sha256d::Hash::from_engine(enc)
            };

            let input = {
                let input = &tx.input[0];
                let value = Value::Explicit(Amount::from_sat(funding_amount).as_sat());
                InputData {
                    previous_output: input.previous_output,
                    script: script.clone(),
                    value,
                    sequence: input.sequence,
                }
            };

            let (principal_repayment_output, tx_fee_output) =
                (tx.output[0].clone(), tx.output[1].clone());

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
                principal_repayment_output,
                tx_fee_output,
                lock_time,
                sighash_type,
            })
        }

        // supports only 1 input atm and SigHashAll only
        fn serialise(&self) -> anyhow::Result<Vec<Vec<u8>>> {
            let sig = self.sig.serialize_der().to_vec();

            let pk = self.pk.serialize().to_vec();

            let tx_version = {
                let mut writer = Vec::new();
                self.tx_version.consensus_encode(&mut writer)?;
                writer
            };

            // input specific values
            let tx_in = {
                let mut writer = Vec::new();
                let InputData {
                    previous_output,
                    script,
                    value,
                    sequence,
                } = &self.input;

                previous_output.consensus_encode(&mut writer)?;
                // TODO: Split
                script.consensus_encode(&mut writer)?;
                value.consensus_encode(&mut writer)?;
                sequence.consensus_encode(&mut writer)?;
                // if txin.has_issuance() {
                //     txin.asset_issuance.consensus_encode(&mut writer)?;
                // }
                writer
            };

            // hashoutputs (only supporting SigHashType::All)
            let (principal_repayment_output, tx_fee_output) = {
                let mut output0 = Vec::new();
                let mut output1 = Vec::new();

                self.principal_repayment_output
                    .consensus_encode(&mut output0)?;
                self.tx_fee_output.consensus_encode(&mut output1)?;
                (output0, output1)
            };

            let lock_time = {
                let mut writer = Vec::new();
                self.lock_time.consensus_encode(&mut writer)?;
                writer
            };

            let sighhash_type = {
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
                tx_in,
                principal_repayment_output,
                tx_fee_output,
                lock_time,
                sighhash_type,
                self.input.script.clone().into_bytes(),
            ])
        }
    }
}
