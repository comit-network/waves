use elements_fun::{
    bitcoin::{Network::Regtest, PrivateKey, PublicKey},
    Address, AddressParams,
};
use rand::thread_rng;
use secp256k1::{SecretKey, SECP256K1};

pub mod states;

pub fn make_keypair() -> (SecretKey, PublicKey) {
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

pub fn make_confidential_address() -> (Address, SecretKey, PublicKey, SecretKey, PublicKey) {
    let (sk, pk) = make_keypair();
    let (blinding_sk, blinding_pk) = make_keypair();

    (
        Address::p2wpkh(&pk, Some(blinding_pk.key), &AddressParams::ELEMENTS),
        sk,
        pk,
        blinding_sk,
        blinding_pk,
    )
}

#[cfg(test)]
mod tests {
    use bitcoin::Amount;
    use elements_fun::{
        bitcoin::{secp256k1::Message, SigHashType},
        bitcoin_hashes::{hash160, hex::FromHex, Hash},
        encode::serialize_hex,
        opcodes,
        script::Builder,
        wally::tx_get_elements_signature_hash,
        OutPoint, Transaction, TxIn, TxOut, UnblindedTxOut,
    };
    use elements_harness::{
        elementd_rpc::{Client, ElementsRpc},
        Elementsd,
    };
    use rand::thread_rng;
    use testcontainers::clients::Cli;

    use crate::make_confidential_address;
    use secp256k1::SECP256K1;

    #[tokio::test]
    async fn sign_transaction_with_two_asset_types() {
        let secp = elements_fun::bitcoin::secp256k1::Secp256k1::new();

        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let litecoin_asset_id = client.issueasset(10.0, 0.0, true).await.unwrap().asset;
        let bitcoin_asset_id = client.get_bitcoin_asset_id().await.unwrap();

        let (
            fund_address_bitcoin,
            fund_sk_bitcoin,
            fund_pk_bitcoin,
            fund_blinding_sk_bitcoin,
            _fund_blinding_pk_bitcoin,
        ) = make_confidential_address();
        let (
            fund_address_litecoin,
            fund_sk_litecoin,
            fund_pk_litecoin,
            fund_blinding_sk_litecoin,
            _fund_blinding_pk_litecoin,
        ) = make_confidential_address();

        let fund_bitcoin_amount = bitcoin::Amount::ONE_BTC;
        let fund_litecoin_amount = bitcoin::Amount::ONE_BTC;

        let fund_bitcoin_txid = client
            .send_asset_to_address(fund_address_bitcoin.clone(), fund_bitcoin_amount, None)
            .await
            .unwrap();

        let fund_litecoin_txid = client
            .send_asset_to_address(
                fund_address_litecoin.clone(),
                fund_litecoin_amount,
                Some(litecoin_asset_id),
            )
            .await
            .unwrap();

        let fund_bitcoin_tx: Transaction = {
            let tx_hex = client.getrawtransaction(fund_bitcoin_txid).await.unwrap();
            elements_fun::encode::deserialize(&Vec::<u8>::from_hex(&tx_hex).unwrap()).unwrap()
        };
        let fund_litecoin_tx: Transaction = {
            let tx_hex = client.getrawtransaction(fund_litecoin_txid).await.unwrap();
            elements_fun::encode::deserialize(&Vec::<u8>::from_hex(&tx_hex).unwrap()).unwrap()
        };
        let fund_bitcoin_vout = fund_bitcoin_tx
            .output
            .iter()
            .position(|output| output.script_pubkey() == &fund_address_bitcoin.script_pubkey())
            .unwrap();
        let fund_litecoin_vout = fund_litecoin_tx
            .output
            .iter()
            .position(|output| output.script_pubkey() == &fund_address_litecoin.script_pubkey())
            .unwrap();

        let redeem_fee = Amount::from_sat(900_000);
        let redeem_amount_bitcoin = fund_bitcoin_amount - redeem_fee;

        let redeem_amount_litecoin = fund_litecoin_amount;

        let (
            redeem_address_bitcoin,
            redeem_sk_bitcoin,
            redeem_pk_bitcoin,
            redeem_blinding_sk_bitcoin,
            _redeem_blinding_pk_bitcoin,
        ) = make_confidential_address();

        let (
            redeem_address_litecoin,
            _redeem_sk_litecoin,
            _redeem_pk_litecoin,
            _redeem_blinding_sk_litecoin,
            _redeem_blinding_pk_litecoin,
        ) = make_confidential_address();

        let tx_out_bitcoin = fund_bitcoin_tx.output[fund_bitcoin_vout]
            .as_confidential()
            .unwrap()
            .clone();
        let tx_out_litecoin = fund_litecoin_tx.output[fund_litecoin_vout]
            .as_confidential()
            .unwrap()
            .clone();

        let UnblindedTxOut {
            asset: unblinded_asset_id_bitcoin,
            asset_blinding_factor: abf_bitcoin,
            value_blinding_factor: vbf_bitcoin,
            value: amount_in_bitcoin,
        } = tx_out_bitcoin.unblind(fund_blinding_sk_bitcoin).unwrap();
        let UnblindedTxOut {
            asset: unblinded_asset_id_litecoin,
            asset_blinding_factor: abf_litecoin,
            value_blinding_factor: vbf_litecoin,
            value: amount_in_litecoin,
        } = tx_out_litecoin.unblind(fund_blinding_sk_litecoin).unwrap();

        #[allow(clippy::cast_possible_truncation)]
        let input_bitcoin = TxIn {
            previous_output: OutPoint {
                txid: fund_bitcoin_txid,
                vout: fund_bitcoin_vout as u32,
            },
            is_pegin: false,
            has_issuance: false,
            script_sig: Default::default(),
            sequence: 0xFFFF_FFFF,
            asset_issuance: Default::default(),
            witness: Default::default(),
        };

        let input_litecoin = TxIn {
            previous_output: OutPoint {
                txid: fund_litecoin_txid,
                vout: fund_litecoin_vout as u32,
            },
            is_pegin: false,
            has_issuance: false,
            script_sig: Default::default(),
            sequence: 0xFFFF_FFFF,
            asset_issuance: Default::default(),
            witness: Default::default(),
        };

        let inputs = [
            (
                unblinded_asset_id_bitcoin,
                amount_in_bitcoin,
                tx_out_bitcoin.asset,
                abf_bitcoin,
                vbf_bitcoin,
            ),
            (
                unblinded_asset_id_litecoin,
                amount_in_litecoin,
                tx_out_litecoin.asset,
                abf_litecoin,
                vbf_litecoin,
            ),
        ];

        let (redeem_txout_bitcoin, redeem_abf_bitcoin, redeem_vbf_bitcoin) =
            TxOut::new_not_last_confidential(
                &mut thread_rng(),
                &SECP256K1,
                redeem_amount_bitcoin.as_sat(),
                redeem_address_bitcoin.clone(),
                bitcoin_asset_id,
                &inputs,
            )
            .unwrap();
        let outputs = [(
            redeem_amount_bitcoin.as_sat(),
            redeem_abf_bitcoin,
            redeem_vbf_bitcoin,
        )];
        let txout_litecoin = TxOut::new_last_confidential(
            &mut thread_rng(),
            &SECP256K1,
            redeem_amount_litecoin.as_sat(),
            redeem_address_litecoin.clone(),
            litecoin_asset_id,
            &inputs,
            &outputs,
        )
        .unwrap();
        let fee = TxOut::new_fee(bitcoin_asset_id, redeem_fee.as_sat());

        let mut redeem_tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![input_bitcoin, input_litecoin],
            output: vec![redeem_txout_bitcoin.clone(), txout_litecoin, fee],
        };

        redeem_tx.input[0].witness.script_witness = {
            let hash = hash160::Hash::hash(&fund_pk_bitcoin.to_bytes());
            let script = Builder::new()
                .push_opcode(opcodes::all::OP_DUP)
                .push_opcode(opcodes::all::OP_HASH160)
                .push_slice(&hash.into_inner())
                .push_opcode(opcodes::all::OP_EQUALVERIFY)
                .push_opcode(opcodes::all::OP_CHECKSIG)
                .into_script();

            let digest = tx_get_elements_signature_hash(
                &redeem_tx,
                0,
                &script,
                &fund_bitcoin_tx.output[fund_bitcoin_vout]
                    .as_confidential()
                    .unwrap()
                    .value,
                1,
                true,
            );

            let sig = secp.sign(
                &Message::from_slice(&digest.into_inner()).unwrap(),
                &fund_sk_bitcoin,
            );

            let mut serialized_signature = sig.serialize_der().to_vec();
            serialized_signature.push(SigHashType::All as u8);

            vec![serialized_signature, fund_pk_bitcoin.to_bytes()]
        };
        redeem_tx.input[1].witness.script_witness = {
            let hash = hash160::Hash::hash(&fund_pk_litecoin.to_bytes());
            let script = Builder::new()
                .push_opcode(opcodes::all::OP_DUP)
                .push_opcode(opcodes::all::OP_HASH160)
                .push_slice(&hash.into_inner())
                .push_opcode(opcodes::all::OP_EQUALVERIFY)
                .push_opcode(opcodes::all::OP_CHECKSIG)
                .into_script();

            let digest = tx_get_elements_signature_hash(
                &redeem_tx,
                1,
                &script,
                &fund_litecoin_tx.output[fund_litecoin_vout]
                    .as_confidential()
                    .unwrap()
                    .value,
                1,
                true,
            );

            let sig = secp.sign(
                &Message::from_slice(&digest.into_inner()).unwrap(),
                &fund_sk_litecoin,
            );

            let mut serialized_signature = sig.serialize_der().to_vec();
            serialized_signature.push(SigHashType::All as u8);

            vec![serialized_signature, fund_pk_litecoin.to_bytes()]
        };

        let tx_hex = serialize_hex(&redeem_tx);
        let _redeem_txid = client.sendrawtransaction(tx_hex).await.unwrap();

        // Verify bitcoin can be spent

        let redeem_vout_bitcoin = redeem_tx
            .output
            .iter()
            .position(|output| output.script_pubkey() == &redeem_address_bitcoin.script_pubkey())
            .unwrap();

        let spend_fee_bitcoin = Amount::from_sat(900_000);
        let spend_amount_bitcoin = redeem_amount_bitcoin - spend_fee_bitcoin;

        let (
            spend_address_bitcoin,
            _spend_sk_bitcoin,
            _spend_pk_bitcoin,
            _spend_blinding_sk_bitcoin,
            _spend_blinding_pk_bitcoin,
        ) = make_confidential_address();

        let tx_out_bitcoin = redeem_tx.output[redeem_vout_bitcoin]
            .as_confidential()
            .unwrap()
            .clone();
        let UnblindedTxOut {
            asset: unblinded_asset_id_bitcoin,
            asset_blinding_factor: abf,
            value_blinding_factor: vbf,
            value: amount_in,
        } = tx_out_bitcoin.unblind(redeem_blinding_sk_bitcoin).unwrap();

        #[allow(clippy::cast_possible_truncation)]
        let spend_input = TxIn {
            previous_output: OutPoint {
                txid: redeem_tx.txid(),
                vout: redeem_vout_bitcoin as u32,
            },
            is_pegin: false,
            has_issuance: false,
            script_sig: Default::default(),
            sequence: 0xFFFF_FFFF,
            asset_issuance: Default::default(),
            witness: Default::default(),
        };

        let inputs = [(
            unblinded_asset_id_bitcoin,
            amount_in,
            tx_out_bitcoin.asset,
            abf,
            vbf,
        )];

        let spend_output = TxOut::new_last_confidential(
            &mut thread_rng(),
            &SECP256K1,
            spend_amount_bitcoin.as_sat(),
            spend_address_bitcoin,
            bitcoin_asset_id,
            &inputs,
            &[],
        )
        .unwrap();

        let fee = TxOut::new_fee(bitcoin_asset_id, spend_fee_bitcoin.as_sat());

        let mut spend_tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![spend_input],
            output: vec![spend_output, fee],
        };

        spend_tx.input[0].witness.script_witness = {
            let hash = hash160::Hash::hash(&redeem_pk_bitcoin.to_bytes());
            let script = Builder::new()
                .push_opcode(opcodes::all::OP_DUP)
                .push_opcode(opcodes::all::OP_HASH160)
                .push_slice(&hash.into_inner())
                .push_opcode(opcodes::all::OP_EQUALVERIFY)
                .push_opcode(opcodes::all::OP_CHECKSIG)
                .into_script();

            let digest = tx_get_elements_signature_hash(
                &spend_tx,
                0,
                &script,
                &redeem_txout_bitcoin.as_confidential().unwrap().value,
                1,
                true,
            );

            let sig = secp.sign(
                &Message::from_slice(&digest.into_inner()).unwrap(),
                &redeem_sk_bitcoin,
            );

            let mut serialized_signature = sig.serialize_der().to_vec();
            serialized_signature.push(SigHashType::All as u8);

            vec![serialized_signature, redeem_pk_bitcoin.to_bytes()]
        };

        let tx_hex = serialize_hex(&spend_tx);
        let _txid = client.sendrawtransaction(tx_hex).await.unwrap();
    }
}
