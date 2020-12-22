use anyhow::{Context, Result};
use elements_fun::{
    bitcoin,
    bitcoin::{Amount, Network, PrivateKey, PublicKey},
    encode::serialize_hex,
    hashes::{hash160, Hash},
    opcodes,
    script::Builder,
    sighash::SigHashCache,
    Address, AddressParams, OutPoint, SigHashType, Transaction, TxIn, TxOut, Txid, UnblindedTxOut,
};
use elements_harness::{elementd_rpc::ElementsRpc, Client, Elementsd};
use secp256k1::{rand::thread_rng, Message, SecretKey, SECP256K1};
use swap::{Alice0, Bob0};
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

    let asset_id_alice = client.get_bitcoin_asset_id().await.unwrap();
    let asset_id_bob = client.issueasset(10.0, 0.0, true).await.unwrap().asset;

    // fund keypairs and addresses
    let (
        fund_address_alice,
        fund_sk_alice,
        _fund_pk_alice,
        fund_blinding_sk_alice,
        _fund_blinding_pk_alice,
    ) = make_confidential_address();
    let (fund_address_bob, fund_sk_bob, _fund_pk_bob, fund_blinding_sk_bob, _fund_blinding_pk_bob) =
        make_confidential_address();

    // redeem keypairs and addresses
    let (
        final_address_alice,
        _final_sk_alice,
        _final_pk_alice,
        final_blinding_sk_alice,
        _final_blinding_pk_alice,
    ) = make_confidential_address();
    let (
        final_address_bob,
        final_sk_bob,
        _final_pk_bob,
        final_blinding_sk_bob,
        _final_blinding_pk_bob,
    ) = make_confidential_address();

    // change keypairs and addresses
    let (
        change_address_alice,
        change_sk_alice,
        _change_pk_alice,
        change_blinding_sk_alice,
        _change_blinding_pk_alice,
    ) = make_confidential_address();
    let (
        change_address_bob,
        _change_sk_bob,
        _change_pk_bob,
        _change_blinding_sk_bob,
        _change_blinding_pk_bob,
    ) = make_confidential_address();

    // initial funding
    let fund_amount_alice = bitcoin::Amount::ONE_BTC;
    let fund_amount_bob = bitcoin::Amount::ONE_BTC;

    let fund_alice_txid = client
        .send_asset_to_address(&fund_address_alice, fund_amount_alice, Some(asset_id_alice))
        .await
        .unwrap();

    let fund_bob_txid = client
        .send_asset_to_address(&fund_address_bob, fund_amount_bob, Some(asset_id_bob))
        .await
        .unwrap();

    let amount_alice = bitcoin::Amount::from_sat(50_000_000);
    let amount_bob = bitcoin::Amount::from_sat(25_000_000);
    let fee = bitcoin::Amount::from_sat(900_000);

    let input_alice = extract_input(
        &client.get_raw_transaction(fund_alice_txid).await.unwrap(),
        fund_address_alice,
    )
    .unwrap();

    let input_bob = extract_input(
        &client.get_raw_transaction(fund_bob_txid).await.unwrap(),
        fund_address_bob,
    )
    .unwrap();

    let alice = Alice0::new(
        amount_alice,
        amount_bob,
        input_alice,
        fund_sk_alice,
        fund_blinding_sk_alice,
        asset_id_bob,
        final_address_alice.clone(),
        final_blinding_sk_alice,
        change_address_alice.clone(),
        change_blinding_sk_alice,
        fee,
    );

    let bob = Bob0::new(
        amount_alice,
        amount_bob,
        input_bob,
        fund_blinding_sk_bob,
        asset_id_alice,
        final_address_bob.clone(),
        change_address_bob.clone(),
    );

    let message0 = alice.compose();
    let bob1 = bob
        .interpret(&mut thread_rng(), SECP256K1, message0)
        .unwrap();
    let message1 = bob1
        .compose(async { bob1.sign_with_key(&fund_sk_bob) })
        .await
        .unwrap();

    let tx = alice.interpret(message1).unwrap();
    let _txid = client.send_raw_transaction(&tx).await.unwrap();

    let (final_output_bob, _) = extract_input(&tx, final_address_bob).unwrap();
    let _txid = move_output_to_wallet(
        &client,
        final_output_bob,
        final_sk_bob,
        final_blinding_sk_bob,
    )
    .await
    .unwrap();

    let (change_output_alice, _) = extract_input(&tx, change_address_alice).unwrap();
    let _txid = move_output_to_wallet(
        &client,
        change_output_alice,
        change_sk_alice,
        change_blinding_sk_alice,
    )
    .await
    .unwrap();
}

// TODO: Only works with Bitcoin. Support other assets
async fn move_output_to_wallet(
    client: &Client,
    previous_output: OutPoint,
    previous_output_sk: SecretKey,
    previous_output_blinding_sk: SecretKey,
) -> Result<Txid> {
    #[allow(clippy::cast_possible_truncation)]
    let input = TxIn {
        previous_output,
        is_pegin: false,
        has_issuance: false,
        script_sig: Default::default(),
        sequence: 0xFFFF_FFFF,
        asset_issuance: Default::default(),
        witness: Default::default(),
    };

    let previous_output_tx = client.get_raw_transaction(previous_output.txid).await?;
    let previous_output = previous_output_tx.output[previous_output.vout as usize].clone();

    let txout = previous_output
        .as_confidential()
        .context("not a confidential txout")?
        .clone();

    let UnblindedTxOut {
        asset: asset_id,
        asset_blinding_factor: abf_in,
        value_blinding_factor: vbf_in,
        value: amount_in,
    } = txout.unblind(SECP256K1, previous_output_blinding_sk)?;

    let fee = 900_000;
    let amount_out = Amount::from_sat(amount_in - fee);

    let move_address = client.getnewaddress().await?;

    let inputs = [(asset_id, amount_in, txout.asset, abf_in, vbf_in)];

    let output = TxOut::new_last_confidential(
        &mut thread_rng(),
        &SECP256K1,
        amount_out.as_sat(),
        move_address,
        asset_id,
        &inputs,
        &[],
    )?;

    let fee = TxOut::new_fee(asset_id, fee);

    let mut tx = Transaction {
        version: 2,
        lock_time: 0,
        input: vec![input],
        output: vec![output, fee],
    };

    let previous_output_pk = PublicKey::from_private_key(
        &SECP256K1,
        &PrivateKey {
            compressed: true,
            network: Network::Regtest,
            key: previous_output_sk,
        },
    );

    tx.input[0].witness.script_witness = {
        let hash = hash160::Hash::hash(&previous_output_pk.to_bytes());
        let script = Builder::new()
            .push_opcode(opcodes::all::OP_DUP)
            .push_opcode(opcodes::all::OP_HASH160)
            .push_slice(&hash.into_inner())
            .push_opcode(opcodes::all::OP_EQUALVERIFY)
            .push_opcode(opcodes::all::OP_CHECKSIG)
            .into_script();

        let sighash =
            SigHashCache::new(&tx).segwitv0_sighash(0, &script, txout.value, SigHashType::All);

        let sig = SECP256K1.sign(&Message::from(sighash), &previous_output_sk);

        let mut serialized_signature = sig.serialize_der().to_vec();
        serialized_signature.push(SigHashType::All as u8);

        vec![serialized_signature, previous_output_pk.to_bytes()]
    };

    let tx_hex = serialize_hex(&tx);
    let txid = client.sendrawtransaction(tx_hex).await?;

    Ok(txid)
}

fn extract_input(tx: &Transaction, address: Address) -> Result<(OutPoint, TxOut)> {
    let vout = tx
        .output
        .iter()
        .position(|output| output.script_pubkey() == &address.script_pubkey())
        .context("Tx doesn't pay to address")?;

    let outpoint = OutPoint {
        txid: tx.txid(),
        vout: vout as u32,
    };
    let tx_out = tx.output[vout].clone();
    Ok((outpoint, tx_out))
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

fn make_confidential_address() -> (Address, SecretKey, PublicKey, SecretKey, PublicKey) {
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
