#[cfg(test)]
mod tests {
    use anyhow::Result;
    use elements::confidential::{Asset, Value};
    use elements::encode::Encodable;
    use elements::opcodes::all::*;
    use elements::opcodes::OP_TRUE;
    use elements::script::Builder;
    use elements::secp256k1::rand::thread_rng;
    use elements::secp256k1::{SecretKey, Signature, SECP256K1};
    use elements::sighash::SigHashCache;
    use elements::{
        bitcoin::hashes::{hash160, sha256d, Hash},
        confidential,
    };
    use elements::{bitcoin::util::psbt::serialize::Serialize, AssetIssuance};
    use elements::{
        bitcoin::{Amount, Network, PrivateKey, PublicKey},
        TxOutWitness,
    };
    use elements::{
        Address, AddressParams, OutPoint, Script, SigHashType, Transaction, TxIn, TxInWitness,
        TxOut,
    };
    use elements_harness::Client;
    use elements_harness::{elementd_rpc::ElementsRpc, Elementsd};
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
        let asset_id_usdt = client.issueasset(40.0, 0.0, false).await.unwrap().asset;

        let (_lender_sk, lender_pk) = make_keypair();
        let lender_address = Address::p2wpkh(&lender_pk, None, &AddressParams::ELEMENTS);

        let (borrower_sk, borrower_pk) = make_keypair();
        let borrower_address = Address::p2wpkh(&borrower_pk, None, &AddressParams::ELEMENTS);

        let principal_amount = 200_000_000;
        let collateral_amount = 1_000_000;
        let tx_fee = 100_000;

        let (repayment_output, repayment_output_bytes) = {
            let txout = TxOut {
                asset: Asset::Explicit(asset_id_usdt),
                value: Value::Explicit(Amount::from_sat(principal_amount).as_sat()),
                nonce: Default::default(),
                script_pubkey: lender_address.script_pubkey(),
                witness: Default::default(),
            };

            let mut res = Vec::new();
            txout.consensus_encode(&mut res).unwrap();

            (txout, res)
        };

        // create covenants script
        let script = Builder::new()
            .push_opcode(OP_IF)
            .push_opcode(OP_DEPTH)
            .push_opcode(OP_1SUB)
            .push_opcode(OP_PICK)
            .push_opcode(OP_PUSHNUM_1)
            .push_opcode(OP_CAT)
            .push_slice(&borrower_pk.serialize())
            .push_opcode(OP_CHECKSIGVERIFY)
            .push_slice(repayment_output_bytes.as_slice())
            .push_opcode(OP_2ROT)
            .push_int(5)
            .push_opcode(OP_ROLL)
            .push_opcode(OP_CAT)
            .push_opcode(OP_CAT)
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
            .push_opcode(OP_CAT)
            .push_opcode(OP_CAT)
            .push_opcode(OP_CAT)
            .push_opcode(OP_CAT)
            .push_opcode(OP_CAT)
            .push_opcode(OP_SHA256)
            .push_opcode(OP_SWAP)
            .push_opcode(OP_CHECKSIGFROMSTACK)
            .push_opcode(OP_ELSE)
            .push_int(10000)
            .push_opcode(OP_CLTV)
            .push_opcode(OP_DROP)
            .push_opcode(OP_DUP)
            .push_slice(&lender_pk.serialize())
            .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_ENDIF)
            .into_script();
        let address = Address::p2wsh(&script, None, &AddressParams::ELEMENTS);

        // borrower locks up the collateral (TODO: also lender pays principal to borrower)
        let collateral_value = Amount::from_sat(collateral_amount);
        let txid = client
            .send_asset_to_address(&address, collateral_value, None)
            .await
            .unwrap();

        // construct collateral input
        let tx = client.get_raw_transaction(txid).await.unwrap();
        let vout = tx
            .output
            .iter()
            .position(|o| o.script_pubkey == address.script_pubkey())
            .unwrap() as u32;

        let collateral_input = TxIn {
            previous_output: OutPoint { txid, vout },
            is_pegin: false,
            has_issuance: false,
            script_sig: Default::default(),
            sequence: 0,
            asset_issuance: Default::default(),
            witness: Default::default(),
        };

        // construct repayment input and repayment change output
        let (
            repayment_input,
            repayment_change,
            repayment_input_sk,
            repayment_input_pk,
            repayment_input_amount,
        ) = {
            let (sk, pk) = make_keypair();
            let address = Address::p2wpkh(&pk, None, &AddressParams::ELEMENTS);
            let txid = client
                .send_asset_to_address(
                    &address,
                    Amount::from_btc(40.0).unwrap(),
                    Some(asset_id_usdt),
                )
                .await
                .unwrap();

            let tx = client.get_raw_transaction(txid).await.unwrap();
            let vout = tx
                .output
                .iter()
                .position(|out| {
                    out.asset.is_explicit() && out.asset.explicit().unwrap() == asset_id_usdt
                })
                .unwrap();
            let amount = tx.output[vout].value.explicit().unwrap();

            let input = TxIn {
                previous_output: OutPoint {
                    txid,
                    vout: vout as u32,
                },
                is_pegin: false,
                has_issuance: false,
                script_sig: Script::default(),
                sequence: 0,
                asset_issuance: AssetIssuance::default(),
                witness: TxInWitness::default(),
            };

            let address = client.getnewaddress().await.unwrap();
            let change_output = TxOut {
                asset: confidential::Asset::Explicit(asset_id_usdt),
                value: confidential::Value::Explicit(amount - principal_amount),
                nonce: confidential::Nonce::Null,
                script_pubkey: address.script_pubkey(),
                witness: TxOutWitness::default(),
            };

            (input, change_output, sk, pk, amount)
        };

        let collateral_output = TxOut {
            asset: Asset::Explicit(asset_id_lbtc),
            value: Value::Explicit(Amount::from_sat(collateral_amount - tx_fee).as_sat()),
            nonce: Default::default(),
            script_pubkey: borrower_address.script_pubkey(),
            witness: Default::default(),
        };

        let tx_fee_output = TxOut {
            asset: Asset::Explicit(asset_id_lbtc),
            value: Value::Explicit(Amount::from_sat(tx_fee).as_sat()),
            nonce: Default::default(),
            script_pubkey: Default::default(),
            witness: Default::default(),
        };

        // borrower repays the principal to get back the collateral
        let mut tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![collateral_input, repayment_input],
            output: vec![
                repayment_output,
                collateral_output,
                repayment_change,
                tx_fee_output,
            ],
        };

        // fulfill collateral input covenant script
        {
            let sighash = SigHashCache::new(&tx).segwitv0_sighash(
                0,
                &script.clone(),
                Value::Explicit(Amount::from_sat(collateral_amount).as_sat()),
                SigHashType::All,
            );

            let sig = SECP256K1.sign(&elements::secp256k1::Message::from(sighash), &borrower_sk);

            tx.input[0].witness = TxInWitness {
                amount_rangeproof: vec![],
                inflation_keys_rangeproof: vec![],
                script_witness: RepaymentWitnessStack::new(
                    sig,
                    borrower_pk,
                    collateral_amount,
                    &tx,
                    script,
                )
                .unwrap()
                .serialise()
                .unwrap(),
                pegin_witness: vec![],
            };
        };

        // sign repayment input
        {
            let hash = hash160::Hash::hash(&repayment_input_pk.serialize());
            let script = Builder::new()
                .push_opcode(OP_DUP)
                .push_opcode(OP_HASH160)
                .push_slice(&hash.into_inner())
                .push_opcode(OP_EQUALVERIFY)
                .push_opcode(OP_CHECKSIG)
                .into_script();

            let sighash = SigHashCache::new(&tx).segwitv0_sighash(
                1,
                &script,
                Value::Explicit(repayment_input_amount),
                SigHashType::All,
            );

            let sig = SECP256K1.sign(
                &elements::secp256k1::Message::from(sighash),
                &repayment_input_sk,
            );
            let mut serialized_signature = sig.serialize_der().to_vec();
            serialized_signature.push(SigHashType::All as u8);

            tx.input[1].witness = TxInWitness {
                amount_rangeproof: vec![],
                inflation_keys_rangeproof: vec![],
                script_witness: vec![
                    serialized_signature,
                    repayment_input_pk.serialize().to_vec(),
                ],
                pegin_witness: vec![],
            };
        };

        client.send_raw_transaction(&tx).await.unwrap();
    }

    struct RepaymentWitnessStack {
        sig: Signature,
        pk: PublicKey,
        tx_version: u32,
        hash_prev_out: elements::hashes::sha256d::Hash,
        hash_sequence: elements::hashes::sha256d::Hash,
        hash_issuances: elements::hashes::sha256d::Hash,
        input: InputData,
        other_outputs: Vec<TxOut>,
        lock_time: u32,
        sighash_type: SigHashType,
    }

    struct InputData {
        previous_output: OutPoint,
        script: Script,
        value: confidential::Value,
        sequence: u32,
    }

    impl RepaymentWitnessStack {
        fn new(
            sig: Signature,
            pk: PublicKey,
            collateral_amount: u64,
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
                let input = &tx.input[0];
                let value = Value::Explicit(collateral_amount);
                InputData {
                    previous_output: input.previous_output,
                    script,
                    value,
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

        // TODO: Currently specific to 1 input, 2 outputs and sighashall
        fn serialise(&self) -> anyhow::Result<Vec<Vec<u8>>> {
            let if_flag = 0x01;

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

                for txout in self.other_outputs.iter() {
                    let mut output = Vec::new();
                    txout.consensus_encode(&mut output)?;
                    other_outputs.push(output)
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
                other_outputs[0].clone(),
                other_outputs[1].clone(),
                other_outputs[2].clone(),
                lock_time,
                sighash_type,
                vec![if_flag],
                self.input.script.clone().into_bytes(),
            ])
        }
    }
}
