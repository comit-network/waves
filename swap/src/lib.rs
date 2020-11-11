#[cfg(test)]
mod tests {
    use ecdsa_fun::{nonce::Deterministic, ECDSA};
    use elements::{
        bitcoin::{
            blockdata::{opcodes, script::Builder},
            PublicKey, Script, SigHashType,
        },
        bitcoin_hashes::{hash160, hex::FromHex, Hash},
        confidential::{Asset, Nonce, Value},
        encode::serialize_hex,
        Address, AddressParams, AssetIssuance, OutPoint, Transaction, TxIn, TxInWitness, TxOut,
        TxOutWitness,
    };
    use elements_harness::{elementd_rpc::Client, elementd_rpc::ElementsRpc, Elementsd};
    use rand::rngs::OsRng;
    use sha2::Sha256;
    use testcontainers::clients::Cli;
    use wally::tx_get_elements_signature_hash;

    #[tokio::test]
    async fn sign_transaction_from_local_address() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let labels = client.dumpassetlabels().await.unwrap();
        let bitcoin_asset_id = labels.get("bitcoin").unwrap();

        let sk = ecdsa_fun::fun::Scalar::random(&mut OsRng);

        let ecdsa = ECDSA::<Deterministic<Sha256>>::default();
        let pk = ecdsa.verification_key_for(&sk);
        let pk = PublicKey::from_slice(&pk.to_bytes()).unwrap();

        let address = Address::p2wpkh(&pk, None, &AddressParams::ELEMENTS);
        let amount = bitcoin::Amount::ONE_BTC;

        let txid = client
            .sendtoaddress(address.clone(), amount.as_btc())
            .await
            .unwrap();
        let tx_hex = client.getrawtransaction(txid).await.unwrap();

        let tx: Transaction =
            elements::encode::deserialize(&Vec::<u8>::from_hex(&tx_hex).unwrap()).unwrap();
        let vout = tx
            .output
            .iter()
            .position(|output| output.script_pubkey == address.script_pubkey())
            .unwrap();

        #[allow(clippy::cast_possible_truncation)]
        let input = TxIn {
            previous_output: OutPoint {
                txid,
                vout: vout as u32,
            },
            is_pegin: false,
            has_issuance: false,
            script_sig: Script::default(),
            sequence: 0xFFFF_FFFF,
            asset_issuance: AssetIssuance::default(),
            witness: TxInWitness::default(),
        };

        let fee = 900_000u64;

        let new_address = client.getnewaddress().await.unwrap();
        let output = TxOut {
            asset: Asset::Explicit(*bitcoin_asset_id),
            value: Value::Explicit(amount.as_sat() - fee),
            nonce: Nonce::Null,
            script_pubkey: new_address.script_pubkey(),
            witness: TxOutWitness::default(),
        };
        let fee = TxOut {
            asset: Asset::Explicit(*bitcoin_asset_id),
            value: Value::Explicit(fee),
            nonce: Nonce::Null,
            script_pubkey: Script::default(),
            witness: TxOutWitness::default(),
        };

        let mut tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![input],
            output: vec![output, fee],
        };

        let script = address.script_pubkey();
        dbg!(&script);

        let hash = hash160::Hash::hash(&pk.to_bytes());
        let script = Builder::new()
            .push_opcode(opcodes::all::OP_DUP)
            .push_opcode(opcodes::all::OP_HASH160)
            .push_slice(&hash.into_inner())
            .push_opcode(opcodes::all::OP_EQUALVERIFY)
            .push_opcode(opcodes::all::OP_CHECKSIG)
            .into_script();
        dbg!(&script);

        let digest = tx_get_elements_signature_hash(
            &tx,
            0,
            &script,
            &Value::Explicit(amount.as_sat()),
            1,
            true,
        );

        let sig = ecdsa.sign(&sk, &digest.into_inner());
        let sig: bitcoin::secp256k1::Signature = sig.into();
        dbg!(&sig);

        let mut serialized_signature = sig.serialize_der().to_vec();
        serialized_signature.push(SigHashType::All as u8);
        tx.input[0].witness.script_witness = vec![serialized_signature, pk.to_bytes()];

        let tx_hex = serialize_hex(&tx);
        let _tx = client.sendrawtransaction(tx_hex).await.unwrap();
    }
}
