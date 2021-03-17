#[cfg(test)]
mod tests {
    use elements::bitcoin::{Amount, Network, PrivateKey, PublicKey};
    use elements::confidential::{Asset, Value};
    use elements::opcodes::all::{
        OP_CAT, OP_CHECKSIGFROMSTACK, OP_CHECKSIGVERIFY, OP_OVER, OP_PICK, OP_SHA256,
    };
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
        // <signature>
        // <sigTransactionData>

        //     OP_OVER OP_SHA256 <pubKey>
        //     2 OP_PICK 1 OP_CAT OP_OVER
        //     OP_CHECKSIGVERIFY
        //     OP_CHECKSIGFROMSTACKVERIFY
        let covenants = Builder::new()
            .push_opcode(OP_OVER)
            .push_opcode(OP_SHA256)
            .push_slice(&pk.key.serialize())
            .push_int(2)
            .push_opcode(OP_PICK)
            .push_int(1)
            .push_opcode(OP_CAT)
            .push_opcode(OP_OVER)
            .push_opcode(OP_CHECKSIGVERIFY)
            .push_opcode(OP_CHECKSIGFROMSTACK)
            .into_script();

        let address = Address::p2wsh(&covenants, None, &AddressParams::ELEMENTS);

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
            &covenants,
            Value::Explicit(Amount::from_sat(funding_amount).as_sat()),
            SigHashType::All,
        );

        let sig = SECP256K1.sign(&elements::secp256k1::Message::from(sighash), &sk);

        let mut serialized_signature = sig.serialize_der().to_vec();
        serialized_signature.push(SigHashType::All as u8);

        tx.input[0].witness = TxInWitness {
            amount_rangeproof: vec![],
            inflation_keys_rangeproof: vec![],
            script_witness: vec![serialized_signature, sighash.to_vec()],
            pegin_witness: vec![],
        };

        client.send_raw_transaction(&tx).await.unwrap();
    }

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
}
