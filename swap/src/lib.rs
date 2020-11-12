#[cfg(test)]
mod tests {
    use elements::bitcoin::secp256k1::Message;
    use elements::bitcoin::secp256k1::PublicKey as SecpPublicKey;
    use elements::bitcoin::secp256k1::SecretKey;
    use elements::bitcoin::Network::Regtest;
    use elements::bitcoin::PrivateKey;
    use elements::{bitcoin::{
        blockdata::{opcodes, script::Builder},
        PublicKey, Script, SigHashType,
    }, bitcoin_hashes::{hash160, hex::FromHex, Hash}, confidential::{Asset, Nonce, Value}, encode::serialize_hex, Address, AddressParams, AssetIssuance, OutPoint, Transaction, TxIn, TxInWitness, TxOut, TxOutWitness, AssetId};
    use elements_harness::{elementd_rpc::Client, elementd_rpc::ElementsRpc, Elementsd};
    use rand::thread_rng;
    use testcontainers::clients::Cli;
    use wally::{asset_generator_from_bytes, asset_rangeproof, asset_surjectionproof, asset_unblind, asset_value_commitment, tx_get_elements_signature_hash, asset_final_vbf};
    use secp256k1::SECP256K1;

    #[tokio::test]
    async fn sign_transaction_from_local_address_non_confidential() {
        let secp = elements::bitcoin::secp256k1::Secp256k1::new();

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

        let sk = SecretKey::new(&mut thread_rng());
        let pk = PublicKey::from_private_key(
            &secp,
            &PrivateKey {
                compressed: true,
                network: Regtest,
                key: sk,
            },
        );

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

        let hash = hash160::Hash::hash(&pk.to_bytes());
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
            &Value::Explicit(amount.as_sat()),
            1,
            true,
        );

        let sig = secp.sign(&Message::from_slice(&digest.into_inner()).unwrap(), &sk);

        let mut serialized_signature = sig.serialize_der().to_vec();
        serialized_signature.push(SigHashType::All as u8);
        tx.input[0].witness.script_witness = vec![serialized_signature, pk.to_bytes()];

        let tx_hex = serialize_hex(&tx);
        let _tx = client.sendrawtransaction(tx_hex).await.unwrap();
    }

    #[tokio::test]
    async fn sign_transaction_from_local_address_confidential() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let labels = client.dumpassetlabels().await.unwrap();
        let bitcoin_asset_tag = "bitcoin";
        let bitcoin_asset_id = labels.get(bitcoin_asset_tag).unwrap();
        let bitcoin_asset_id_bytes = bitcoin_asset_id.into_inner().0;

        let fund_sk = SecretKey::new(&mut thread_rng());
        let fund_pk = PublicKey::from_private_key(
            &SECP256K1,
            &PrivateKey {
                compressed: true,
                network: Regtest,
                key: fund_sk,
            },
        );

        let fund_blinding_sk = SecretKey::new(&mut thread_rng());
        let fund_blinding_pk = PublicKey::from_private_key(
            &SECP256K1,
            &PrivateKey {
                compressed: true,
                network: Regtest,
                key: fund_blinding_sk,
            },
        );

        let fund_address = Address::p2wpkh(
            &fund_pk,
            Some(fund_blinding_pk.key),
            &AddressParams::ELEMENTS,
        );
        let fund_amount = bitcoin::Amount::ONE_BTC;

        let fund_txid = client
            .sendtoaddress(fund_address.clone(), fund_amount.as_btc())
            .await
            .unwrap();

        let fund_tx: Transaction = {
            let tx_hex = client.getrawtransaction(fund_txid).await.unwrap();
            elements::encode::deserialize(&Vec::<u8>::from_hex(&tx_hex).unwrap()).unwrap()
        };
        let fund_vout = fund_tx
            .output
            .iter()
            .position(|output| output.script_pubkey == fund_address.script_pubkey())
            .unwrap();

        #[allow(clippy::cast_possible_truncation)]
        let input = TxIn {
            previous_output: OutPoint {
                txid: fund_txid,
                vout: fund_vout as u32,
            },
            is_pegin: false,
            has_issuance: false,
            script_sig: Script::default(),
            sequence: 0xFFFF_FFFF,
            asset_issuance: AssetIssuance::default(),
            witness: TxInWitness::default(),
        };

        let redeem_fee = 900_000u64;
        let redeem_amount = fund_amount.as_sat() - redeem_fee;

        // unused because we only have a single output
        let _redeem_vbf = SecretKey::new(&mut thread_rng());

        let redeem_abf = SecretKey::new(&mut thread_rng());
        let redeem_asset = asset_generator_from_bytes(&bitcoin_asset_id_bytes, redeem_abf.as_ref());

        let redeem_sk = SecretKey::new(&mut thread_rng());
        let redeem_pk = PublicKey::from_private_key(
            &SECP256K1,
            &PrivateKey {
                compressed: true,
                network: Regtest,
                key: redeem_sk,
            },
        );

        let redeem_blinding_sk = SecretKey::new(&mut thread_rng());
        let redeem_blinding_pk = PublicKey::from_private_key(
            &SECP256K1,
            &PrivateKey {
                compressed: true,
                network: Regtest,
                key: redeem_blinding_sk,
            },
        );

        let redeem_address = Address::p2wpkh(
            &redeem_pk,
            Some(redeem_blinding_pk.key),
            &AddressParams::ELEMENTS,
        );

        let (_unblinded_asset_in, blinded_asset_in, abf_in, vbf_in, _value_out) = {
            let out = fund_tx.output[fund_vout].clone();
            let range_proof = out.witness.rangeproof;
            let value_commitment = out.value.commitment().unwrap();
            let asset_generator = out.asset.commitment().unwrap();
            let script = out.script_pubkey;
            let sender_nonce = out.nonce.commitment().unwrap();
            let sender_pk = SecpPublicKey::from_slice(&sender_nonce).unwrap();

            let (unblinded_asset, abf, vbf, value_out) = asset_unblind(
                sender_pk,
                fund_blinding_sk,
                range_proof,
                value_commitment.into(),
                script,
                asset_generator.into(),
            )
                .unwrap();

            (unblinded_asset, out.asset.commitment(), abf, vbf, value_out)
        };

        let mut abfs = abf_in.to_vec();
        abfs.extend(redeem_abf.as_ref());

        let asset_final_vbf = asset_final_vbf(vec![fund_amount.as_sat(), redeem_amount], 1, abfs, vbf_in.to_vec());

        let redeem_value_commitment =
            asset_value_commitment(redeem_amount, asset_final_vbf, redeem_asset);

        // NOTE: This could be wrong
        let ephemeral_sk = SecretKey::new(&mut thread_rng());

        let range_proof = asset_rangeproof(
            redeem_amount,
            redeem_blinding_pk.key,
            ephemeral_sk,
            bitcoin_asset_id_bytes,
            *redeem_abf.as_ref(),
            asset_final_vbf,
            redeem_value_commitment,
            &redeem_address.script_pubkey(),
            redeem_asset,
            1,
            0,
            52,
        );


        // NOTE: This is probably wrong
        // NOTE: I think it isn't.
        let nonce_sk = SecretKey::new(&mut thread_rng());
        let nonce = Nonce::Confidential(02, *nonce_sk.as_ref());

        let surjection_proof = asset_surjectionproof(
            bitcoin_asset_id_bytes,
            *redeem_abf.as_ref(),
            redeem_asset,
            *nonce_sk.as_ref(),
            &bitcoin_asset_id_bytes.to_vec(),
            &abf_in.to_vec(),
            &blinded_asset_in.unwrap().to_vec(),
            1,
        );

        let output = TxOut {
            asset: redeem_asset,
            value: redeem_value_commitment,
            nonce,
            script_pubkey: redeem_address.script_pubkey(),
            witness: TxOutWitness {
                surjection_proof,
                rangeproof: range_proof,
            },
        };

        let fee = TxOut {
            asset: Asset::Explicit(*bitcoin_asset_id),
            value: Value::Explicit(redeem_fee),
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

        let hash = hash160::Hash::hash(&fund_pk.to_bytes());
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
            &fund_tx.output[fund_vout].value,
            1,
            true,
        );

        let sig = &SECP256K1.sign(
            &Message::from_slice(&digest.into_inner()).unwrap(),
            &fund_sk,
        );

        let mut serialized_signature = sig.serialize_der().to_vec();
        serialized_signature.push(SigHashType::All as u8);
        tx.input[0].witness.script_witness = vec![serialized_signature, fund_pk.to_bytes()];

        let tx_hex = serialize_hex(&tx);
        let _tx = client.sendrawtransaction(tx_hex).await.unwrap();
    }

    fn make_keypair() -> (SecretKey, PublicKey) {
        let sk = SecretKey::new(&mut thread_rng());
        let pk = PublicKey::from_private_key(
            &SECP256K1,
            &PrivateKey {
                compressed: true,
                network: Regtest,
                key: sk,
            },
        );

        (sk, pk)
    }

    fn make_confidential_address(pk: PublicKey, blinding_key: PublicKey) -> Address {
        Address::p2wpkh(
            &pk,
            Some(blinding_key.key),
            &AddressParams::ELEMENTS,
        )
    }

    fn unblind_asset_from_txout(out: TxOut, sender_blinding_sk: SecretKey) -> (AssetId, [u8; 33], [u8; 32], [u8; 32], u64) {
        let range_proof = out.witness.rangeproof;
        let value_commitment = out.value.commitment().unwrap();
        let asset_generator = out.asset.commitment().unwrap();
        let script = out.script_pubkey;
        let sender_nonce = out.nonce.commitment().unwrap();
        let sender_pk = SecpPublicKey::from_slice(&sender_nonce).unwrap();

        let (unblinded_asset, abf, vbf, value_out) = asset_unblind(
            sender_pk,
            sender_blinding_sk,
            range_proof,
            value_commitment.into(),
            script,
            asset_generator.into(),
        )
            .unwrap();

        (AssetId::from_slice(&unblinded_asset).unwrap(), out.asset.commitment().unwrap(), abf, vbf, value_out)
    }

    #[tokio::test]
    async fn sign_transaction_with_two_confidential_inputs() {
        let secp = elements::bitcoin::secp256k1::Secp256k1::new();

        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let bitcoin_asset_id = client.get_bitcoin_asset_id().await.unwrap();
        let bitcoin_asset_id_bytes = bitcoin_asset_id.into_inner().0;

        let (fund_sk_0, fund_pk_0) = make_keypair();
        let (fund_blinding_sk_0, fund_blinding_pk_0) = make_keypair();

        let (fund_sk_1, fund_pk_1) = make_keypair();
        let (fund_blinding_sk_1, fund_blinding_pk_1) = make_keypair();

        let fund_address_0 = make_confidential_address(fund_pk_0, fund_blinding_pk_0);
        let fund_address_1 = make_confidential_address(fund_pk_1, fund_blinding_pk_1);

        let fund_amount = bitcoin::Amount::ONE_BTC;

        let fund_0_txid = client
            .sendtoaddress(fund_address_0.clone(), fund_amount.as_btc())
            .await
            .unwrap();
        let fund_1_txid = client
            .sendtoaddress(fund_address_1.clone(), fund_amount.as_btc())
            .await
            .unwrap();

        let fund_0_tx: Transaction = {
            let tx_hex = client.getrawtransaction(fund_0_txid).await.unwrap();
            elements::encode::deserialize(&Vec::<u8>::from_hex(&tx_hex).unwrap()).unwrap()
        };
        let fund_1_tx: Transaction = {
            let tx_hex = client.getrawtransaction(fund_1_txid).await.unwrap();
            elements::encode::deserialize(&Vec::<u8>::from_hex(&tx_hex).unwrap()).unwrap()
        };
        let fund_0_vout = fund_0_tx
            .output
            .iter()
            .position(|output| output.script_pubkey == fund_address_0.script_pubkey())
            .unwrap();
        let fund_1_vout = fund_1_tx
            .output
            .iter()
            .position(|output| output.script_pubkey == fund_address_1.script_pubkey())
            .unwrap();

        let redeem_fee = 900_000u64;
        let redeem_amount = fund_amount.as_sat() * 2 - redeem_fee;

        let redeem_abf = SecretKey::new(&mut thread_rng());
        let redeem_asset = asset_generator_from_bytes(&bitcoin_asset_id_bytes, redeem_abf.as_ref());

        let ( redeem_sk, redeem_pk) = make_keypair();
        let (redeem_blinding_sk, redeem_blinding_pk) = make_keypair();

        let redeem_address = make_confidential_address(redeem_pk, redeem_blinding_pk);

        let tx_out_0 = fund_0_tx.output[fund_0_vout].clone();
        let tx_out_1 = fund_1_tx.output[fund_1_vout].clone();

        let (unblinded_asset_id_0, asset_commitment_0, abf_0, vbf_0, amount_in_0) = unblind_asset_from_txout(tx_out_0, fund_blinding_sk_0);
        let (unblinded_asset_id_1, asset_commitment_1, abf_1, vbf_1, amount_in_1) = unblind_asset_from_txout(tx_out_1, fund_blinding_sk_1);

        let abfs = vec![abf_0.to_vec(), abf_1.to_vec(), redeem_abf.as_ref().to_vec()].into_iter().flatten().collect::<Vec<_>>();
        let vbfs = vec![vbf_0.to_vec(), vbf_1.to_vec()].into_iter().flatten().collect::<Vec<_>>();

        let asset_final_vbf = asset_final_vbf(vec![amount_in_0, amount_in_1, redeem_amount], 2, abfs, vbfs);

        let redeem_value_commitment =
            asset_value_commitment(redeem_amount, asset_final_vbf, redeem_asset);

        let range_proof = asset_rangeproof(
            redeem_amount,
            redeem_blinding_pk.key,
            SecretKey::new(&mut thread_rng()),
            bitcoin_asset_id_bytes,
            *redeem_abf.as_ref(),
            asset_final_vbf,
            redeem_value_commitment,
            &redeem_address.script_pubkey(),
            redeem_asset,
            1,
            0,
            52,
        );

        // NOTE: This is probably wrong
        // NOTE: I think it isn't.
        let nonce_sk = SecretKey::new(&mut thread_rng());
        let nonce = Nonce::Confidential(02, *nonce_sk.as_ref());

        let unblinded_assets_in = vec![unblinded_asset_id_0.into_inner().0.to_vec(), unblinded_asset_id_1.into_inner().0.to_vec()].into_iter().flatten().collect::<Vec<_>>();
        let abfs_in = vec![abf_0.to_vec(), abf_1.to_vec()].into_iter().flatten().collect::<Vec<_>>();
        let blinded_assets_in = vec![asset_commitment_0.to_vec(), asset_commitment_1.to_vec()].into_iter().flatten().collect::<Vec<_>>();

        let surjection_proof = asset_surjectionproof(
            bitcoin_asset_id_bytes,
            *redeem_abf.as_ref(),
            redeem_asset,
            *nonce_sk.as_ref(),
            &unblinded_assets_in,
            &abfs_in,
            &blinded_assets_in,
            2,
        );

        #[allow(clippy::cast_possible_truncation)]
        let input_0 = TxIn {
            previous_output: OutPoint {
                txid: fund_0_txid,
                vout: fund_0_vout as u32,
            },
            is_pegin: false,
            has_issuance: false,
            script_sig: Default::default(),
            sequence: 0xFFFF_FFFF,
            asset_issuance: Default::default(),
            witness: Default::default()
        };

        let input_1 = TxIn {
            previous_output: OutPoint {
                txid: fund_1_txid,
                vout: fund_1_vout as u32,
            },
            is_pegin: false,
            has_issuance: false,
            script_sig: Default::default(),
            sequence: 0xFFFF_FFFF,
            asset_issuance: Default::default(),
            witness: Default::default()
        };

        let output = TxOut {
            asset: redeem_asset,
            value: redeem_value_commitment,
            nonce: Nonce::Null, // TODO: This is likely wrong
            script_pubkey: redeem_address.script_pubkey(),
            witness: TxOutWitness {
                surjection_proof,
                rangeproof: range_proof,
            },
        };

        let fee = TxOut {
            asset: Asset::Explicit(bitcoin_asset_id),
            value: Value::Explicit(redeem_fee),
            nonce: Nonce::Null,
            script_pubkey: Script::default(),
            witness: TxOutWitness::default(),
        };

        let mut tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![input_0, input_1],
            output: vec![output, fee],
        };

        tx.input[0].witness.script_witness = {
            let hash = hash160::Hash::hash(&fund_pk_0.to_bytes());
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
                &fund_0_tx.output[fund_0_vout].value,
                1,
                true,
            );

            let sig = secp.sign(
                &Message::from_slice(&digest.into_inner()).unwrap(),
                &fund_sk_0,
            );

            let mut serialized_signature = sig.serialize_der().to_vec();
            serialized_signature.push(SigHashType::All as u8);

            vec![serialized_signature, fund_pk_0.to_bytes()]
        };
        tx.input[1].witness.script_witness = {
            let hash = hash160::Hash::hash(&fund_pk_1.to_bytes());
            let script = Builder::new()
                .push_opcode(opcodes::all::OP_DUP)
                .push_opcode(opcodes::all::OP_HASH160)
                .push_slice(&hash.into_inner())
                .push_opcode(opcodes::all::OP_EQUALVERIFY)
                .push_opcode(opcodes::all::OP_CHECKSIG)
                .into_script();

            let digest = tx_get_elements_signature_hash(
                &tx,
                1,
                &script,
                &fund_1_tx.output[fund_1_vout].value,
                1,
                true,
            );

            let sig = secp.sign(
                &Message::from_slice(&digest.into_inner()).unwrap(),
                &fund_sk_1,
            );

            let mut serialized_signature = sig.serialize_der().to_vec();
            serialized_signature.push(SigHashType::All as u8);

            vec![serialized_signature, fund_pk_1.to_bytes()]
        };

        let tx_hex = serialize_hex(&tx);
        let _tx = client.sendrawtransaction(tx_hex).await.unwrap();
    }
}
