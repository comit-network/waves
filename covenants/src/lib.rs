#[cfg(test)]
mod tests {
    use elements::bitcoin::hashes::{sha256d, Hash};
    use elements::bitcoin::util::psbt::serialize::Serialize;
    use elements::bitcoin::{Amount, Network, PrivateKey, PublicKey};
    use elements::confidential::{Asset, Value};
    use elements::encode::Encodable;
    use elements::opcodes::all::{OP_CAT, OP_CHECKSIGFROMSTACK, OP_SHA256, OP_SWAP};
    use elements::script::Builder;
    use elements::secp256k1::rand::thread_rng;
    use elements::secp256k1::{SecretKey, Signature, SECP256K1};
    use elements::sighash::SigHashCache;
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
            script_witness: create_witness_stack(sig, pk, funding_amount, &tx, script),
            pegin_witness: vec![],
        };

        client.send_raw_transaction(&tx).await.unwrap();
    }

    fn create_witness_stack(
        signature: Signature,
        pk: PublicKey,
        funding_amount: u64,
        transaction: &Transaction,
        script: Script,
    ) -> Vec<Vec<u8>> {
        let pk = pk.serialize().to_vec();
        let signature = signature.serialize_der().to_vec();

        let mut signing_data = Vec::new();
        let value = Value::Explicit(Amount::from_sat(funding_amount).as_sat());
        SigHashCache::new(transaction)
            .encode_segwitv0_signing_data_to(&mut signing_data, 0, &script, value, SigHashType::All)
            .unwrap();

        let (
            tx_version,
            hash_prev_out,
            hash_sequence,
            hash_issuances,
            tx_in,
            tx_outs,
            lock_time,
            sighhash_type,
        ) = create_signing_data(transaction, script.clone(), value).unwrap();

        // assert that we created the correct data:
        let mut tx_data = vec![];
        tx_data.append(&mut tx_version.clone());
        tx_data.append(&mut hash_prev_out.clone());
        tx_data.append(&mut hash_sequence.clone());
        tx_data.append(&mut hash_issuances.clone());
        tx_data.append(&mut tx_in.clone());
        tx_data.append(&mut tx_outs.clone());
        tx_data.append(&mut lock_time.clone());
        tx_data.append(&mut sighhash_type.clone());
        assert_eq!(tx_data, signing_data.clone());

        vec![
            signature,
            pk,
            tx_version,
            hash_prev_out,
            hash_sequence,
            hash_issuances,
            tx_in,
            tx_outs,
            lock_time,
            sighhash_type,
            script.into_bytes(),
        ]
    }

    // supports only 1 input atm and SigHashAll only
    fn create_signing_data(
        tx: &Transaction,
        script: Script,
        value: Value,
    ) -> anyhow::Result<(
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
    )> {
        let tx_version = {
            let mut writer = Vec::new();
            tx.version.consensus_encode(&mut writer)?;
            writer
        };

        let hash_prev_out = {
            let mut enc = sha256d::Hash::engine();
            tx.input[0].previous_output.consensus_encode(&mut enc)?;
            sha256d::Hash::from_engine(enc).to_vec()
        };

        let hash_sequence = {
            let mut enc = sha256d::Hash::engine();
            tx.input[0].sequence.consensus_encode(&mut enc)?;
            sha256d::Hash::from_engine(enc).to_vec()
        };

        let hash_issuances = {
            let mut enc = sha256d::Hash::engine();
            if tx.input[0].has_issuance() {
                tx.input[0].asset_issuance.consensus_encode(&mut enc)?;
            } else {
                0u8.consensus_encode(&mut enc)?;
            }
            sha256d::Hash::from_engine(enc).to_vec()
        };

        // input specific values
        let tx_in = {
            let mut writer = Vec::new();
            let txin = &tx.input[0];

            txin.previous_output.consensus_encode(&mut writer)?;
            script.consensus_encode(&mut writer)?;
            value.consensus_encode(&mut writer)?;
            txin.sequence.consensus_encode(&mut writer)?;
            if txin.has_issuance() {
                txin.asset_issuance.consensus_encode(&mut writer)?;
            }
            writer
        };

        // hashoutputs (only supporting SigHashType::All)
        let tx_outs = {
            let mut enc = sha256d::Hash::engine();
            let output = &tx.output;
            for txout in output {
                txout.consensus_encode(&mut enc)?;
            }
            sha256d::Hash::from_engine(enc).to_vec()
        };

        let lock_time = {
            let mut writer = Vec::new();
            tx.lock_time.consensus_encode(&mut writer)?;
            writer
        };

        let sighhash_type = {
            let mut writer = Vec::new();
            SigHashType::All.as_u32().consensus_encode(&mut writer)?;
            writer
        };

        Ok((
            tx_version,
            hash_prev_out,
            hash_sequence,
            hash_issuances,
            tx_in,
            tx_outs,
            lock_time,
            sighhash_type,
        ))
    }
}
