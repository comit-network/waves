use anyhow::Result;
use bitcoin::Amount;
use elements_fun::bitcoin::secp256k1::PublicKey as SecpPublicKey;
use elements_fun::bitcoin::Network::Regtest;
use elements_fun::bitcoin::PrivateKey;
use elements_fun::bitcoin::PublicKey;
use elements_fun::confidential::{
    AssetBlindingFactor, AssetCommitment, NonceCommitment, ValueBlindingFactor,
};
use elements_fun::wally::asset_generator_from_bytes;
use elements_fun::wally::asset_rangeproof;
use elements_fun::wally::asset_surjectionproof;
use elements_fun::wally::asset_value_commitment;
use elements_fun::AddressParams;
use elements_fun::TxOutWitness;
use elements_fun::{Address, ConfidentialTxOut};
use elements_fun::{AssetId, TxOut};
use rand::thread_rng;
use rand::CryptoRng;
use rand::RngCore;
use secp256k1::SecretKey;
use secp256k1::SECP256K1;

pub mod states;

#[allow(clippy::too_many_arguments)]
fn make_txout<R>(
    rng: &mut R,
    amount: Amount,
    address: Address,
    out_asset_id: AssetId,
    out_abf: AssetBlindingFactor,
    out_vbf: ValueBlindingFactor,
    inputs: &[(AssetId, AssetCommitment, AssetBlindingFactor)],
    sender_ephemeral_sk: SecretKey,
) -> Result<TxOut>
where
    R: RngCore + CryptoRng,
{
    let out_asset_id_bytes = out_asset_id.into_inner().0;

    let out_asset = asset_generator_from_bytes(&out_asset_id_bytes, &out_abf);

    let value_commitment = asset_value_commitment(amount.as_sat(), out_vbf, out_asset);

    let range_proof = asset_rangeproof(
        amount.as_sat(),
        address.blinding_pubkey.unwrap(),
        sender_ephemeral_sk,
        out_asset_id_bytes,
        out_abf,
        out_vbf,
        value_commitment,
        &address.script_pubkey(),
        out_asset,
        1,
        0,
        52,
    );

    let unblinded_assets_in = inputs.iter().map(|(id, _, _)| *id).collect::<Vec<_>>();
    let assets_in = inputs
        .iter()
        .map(|(_, asset, _)| *asset)
        .collect::<Vec<_>>();
    let abfs_in = inputs.iter().map(|(_, _, abf)| *abf).collect::<Vec<_>>();

    let surjection_proof = asset_surjectionproof(
        out_asset_id,
        out_abf,
        out_asset,
        *SecretKey::new(rng).as_ref(),
        &unblinded_assets_in,
        &abfs_in,
        &assets_in,
        inputs.len(),
    );

    let sender_ephemeral_pk = SecpPublicKey::from_secret_key(&SECP256K1, &sender_ephemeral_sk);
    Ok(TxOut::Confidential(ConfidentialTxOut {
        asset: out_asset,
        value: value_commitment,
        nonce: Some(NonceCommitment::from_slice(
            &sender_ephemeral_pk.serialize(),
        )?),
        script_pubkey: address.script_pubkey(),
        witness: TxOutWitness {
            surjection_proof,
            rangeproof: range_proof,
        },
    }))
}

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
    use elements_fun::bitcoin::secp256k1::Message;
    use elements_fun::bitcoin::secp256k1::SecretKey;
    use elements_fun::wally::{asset_final_vbf, tx_get_elements_signature_hash};
    use elements_fun::{
        bitcoin::{
            blockdata::{opcodes, script::Builder},
            SigHashType,
        },
        bitcoin_hashes::{hash160, hex::FromHex, Hash},
        encode::serialize_hex,
        OutPoint, Transaction, TxIn, TxOut, UnblindedTxOut,
    };
    use elements_harness::{elementd_rpc::Client, elementd_rpc::ElementsRpc, Elementsd};
    use rand::thread_rng;
    use testcontainers::clients::Cli;

    use crate::make_confidential_address;
    use crate::make_txout;
    use elements_fun::confidential::{AssetBlindingFactor, ValueBlindingFactor};

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

        let redeem_abf_bitcoin = AssetBlindingFactor::new(&mut thread_rng());
        let redeem_abf_litecoin = AssetBlindingFactor::new(&mut thread_rng());

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

        let abfs = &[
            abf_bitcoin,
            abf_litecoin,
            redeem_abf_bitcoin,
            redeem_abf_litecoin,
        ];

        let vbf_redeem_bitcoin = ValueBlindingFactor::new(&mut thread_rng());
        let vbfs = &[vbf_bitcoin, vbf_litecoin, vbf_redeem_bitcoin];

        let vbf_redeem_litecoin = asset_final_vbf(
            vec![
                amount_in_bitcoin,
                amount_in_litecoin,
                redeem_amount_bitcoin.as_sat(),
                redeem_amount_litecoin.as_sat(),
            ],
            2,
            abfs,
            vbfs,
        );

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

        let inputs = vec![
            (
                unblinded_asset_id_bitcoin,
                tx_out_bitcoin.asset,
                abf_bitcoin,
            ),
            (
                unblinded_asset_id_litecoin,
                tx_out_litecoin.asset,
                abf_litecoin,
            ),
        ];

        let redeem_txout_bitcoin = make_txout(
            &mut thread_rng(),
            redeem_amount_bitcoin,
            redeem_address_bitcoin.clone(),
            bitcoin_asset_id,
            redeem_abf_bitcoin,
            vbf_redeem_bitcoin,
            &inputs,
            SecretKey::new(&mut thread_rng()),
        )
        .unwrap();
        let txout_litecoin = make_txout(
            &mut thread_rng(),
            redeem_amount_litecoin,
            redeem_address_litecoin,
            litecoin_asset_id,
            redeem_abf_litecoin,
            vbf_redeem_litecoin,
            &inputs,
            SecretKey::new(&mut thread_rng()),
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

        let spend_abf_bitcoin = AssetBlindingFactor::new(&mut thread_rng());

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

        let abfs = &[abf, spend_abf_bitcoin];
        let vbfs = &[vbf];

        let spend_vbf_bitcoin = asset_final_vbf(
            vec![amount_in, spend_amount_bitcoin.as_sat()],
            1,
            abfs,
            vbfs,
        );

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

        let inputs = vec![(unblinded_asset_id_bitcoin, tx_out_bitcoin.asset, abf)];

        let spend_output = make_txout(
            &mut thread_rng(),
            spend_amount_bitcoin,
            spend_address_bitcoin,
            bitcoin_asset_id,
            spend_abf_bitcoin,
            spend_vbf_bitcoin,
            &inputs,
            SecretKey::new(&mut thread_rng()),
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
