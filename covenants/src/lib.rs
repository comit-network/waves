#[cfg(test)]
mod tests {
    use elements::bitcoin::util::psbt::serialize::Serialize;
    use elements::bitcoin::{Amount, Network, PrivateKey, PublicKey};
    use elements::confidential::{Asset, Value};
    use elements::opcodes::all::{OP_CAT, OP_CHECKSIGFROMSTACK, OP_SHA256, OP_SWAP};
    use elements::script::Builder;
    use elements::secp256k1::rand::thread_rng;
    use elements::secp256k1::{SecretKey, SECP256K1};
    use elements::sighash::SigHashCache;
    use elements::{
        Address, AddressParams, OutPoint, SigHashType, Transaction, TxIn, TxInWitness, TxOut,
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

        let (sk, pk) = make_keypair();
        let script = Builder::new()
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
        let serialized_signature = sig.serialize_der().to_vec();

        let mut signing_data = Vec::new();
        SigHashCache::new(&tx)
            .encode_segwitv0_signing_data_to(
                &mut signing_data,
                0,
                &script,
                Value::Explicit(Amount::from_sat(funding_amount).as_sat()),
                SigHashType::All,
            )
            .unwrap();

        let tx_data1 = signing_data[..80].to_vec();
        let tx_data2 = signing_data[80..160].to_vec();
        let tx_data3 = signing_data[160..].to_vec();

        tx.input[0].witness = TxInWitness {
            amount_rangeproof: vec![],
            inflation_keys_rangeproof: vec![],
            script_witness: vec![
                serialized_signature,
                pk.serialize().to_vec(),
                tx_data1,
                tx_data2,
                tx_data3,
                script.into_bytes(),
            ],
            pegin_witness: vec![],
        };

        client.send_raw_transaction(&tx).await.unwrap();
    }
}
