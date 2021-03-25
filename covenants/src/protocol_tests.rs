#[cfg(test)]
mod tests {
    use crate::{Borrower0, Lender0};
    use anyhow::Result;
    use bitcoin_hashes::Hash;
    use elements::{
        bitcoin::Amount,
        hashes::hash160,
        opcodes,
        script::Builder,
        secp256k1::{rand::thread_rng, SecretKey, SECP256K1},
        sighash::SigHashCache,
        AssetId, OutPoint, Script, SigHashType, TxIn,
    };
    use elements_harness::{elementd_rpc::ElementsRpc, Elementsd};
    use secp256k1_zkp::Message;
    use testcontainers::clients::Cli;

    #[tokio::test]
    async fn borrow_and_repay() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                elements_harness::Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let master_blinding_key = client.dumpmasterblindingkey().await.unwrap();
        let master_blinding_key = hex::decode(master_blinding_key).unwrap();

        let bitcoin_asset_id = client.get_bitcoin_asset_id().await.unwrap();
        let usdt_asset_id = client.issueasset(40.0, 0.0, false).await.unwrap().asset;

        let address = client
            .get_new_address(Some("blech32".into()))
            .await
            .unwrap();
        client
            .send_asset_to_address(&address, Amount::from_btc(5.0).unwrap(), None)
            .await
            .unwrap();
        let miner_address = client
            .get_new_address(Some("blech32".into()))
            .await
            .unwrap();
        client.generatetoaddress(10, &miner_address).await.unwrap();

        let (borrower, _borrower_address) = {
            let collateral_amount = Amount::ONE_BTC;
            let collateral_inputs = find_inputs(&client, bitcoin_asset_id, collateral_amount * 2)
                .await
                .unwrap();
            let address = client
                .get_new_address(Some("blech32".into()))
                .await
                .unwrap();
            let address_blinding_sk =
                derive_blinding_key(master_blinding_key.clone(), address.script_pubkey()).unwrap();

            let timelock = 0;

            let borrower = Borrower0::new(
                address.clone(),
                address_blinding_sk,
                collateral_amount,
                collateral_inputs,
                Amount::ONE_SAT,
                timelock,
                bitcoin_asset_id,
                usdt_asset_id,
            )
            .unwrap();

            (borrower, address)
        };

        // TODO: Use a separate wallet per actor. Using the same wallet is confusing and bug-prone.
        let (lender, _lender_address) = {
            let address = client
                .get_new_address(Some("blech32".into()))
                .await
                .unwrap();

            let principal_inputs =
                find_inputs(&client, usdt_asset_id, Amount::from_btc(2.0).unwrap())
                    .await
                    .unwrap();

            let lender = Lender0::new(
                &SECP256K1,
                bitcoin_asset_id,
                usdt_asset_id,
                principal_inputs,
                address.clone(),
            )
            .unwrap();

            (lender, address)
        };

        let loan_request = borrower.loan_request();

        let lender = lender
            .interpret(&mut thread_rng(), &SECP256K1, loan_request)
            .unwrap();
        let loan_response = lender.loan_response();

        let borrower = borrower.interpret(&SECP256K1, loan_response).unwrap();
        let loan_transaction = borrower
            .sign({
                let client = client.clone();
                |transaction| async move { client.sign_raw_transaction(&transaction).await }
            })
            .await
            .unwrap();

        let loan_transaction = lender
            .finalise_loan(loan_transaction, {
                let client = client.clone();
                |transaction| async move { client.sign_raw_transaction(&transaction).await }
            })
            .await
            .unwrap();

        client
            .send_raw_transaction(&loan_transaction)
            .await
            .unwrap();

        client.generatetoaddress(1, &miner_address).await.unwrap();

        let loan_repayment_transaction = borrower
            .loan_repayment_transaction(
                &mut thread_rng(),
                &SECP256K1,
                {
                    let client = client.clone();
                    |amount, asset| async move { find_inputs(&client, asset, amount).await }
                },
                {
                    let client = client.clone();
                    |mut tx, index, address, value| async move {
                        let sk = client.dump_private_key(&address).await?;
                        let pk = secp256k1_zkp::PublicKey::from_secret_key(&SECP256K1, &sk);

                        let hash = hash160::Hash::hash(&pk.serialize());
                        let script = Builder::new()
                            .push_opcode(opcodes::all::OP_DUP)
                            .push_opcode(opcodes::all::OP_HASH160)
                            .push_slice(&hash.into_inner())
                            .push_opcode(opcodes::all::OP_EQUALVERIFY)
                            .push_opcode(opcodes::all::OP_CHECKSIG)
                            .into_script();

                        let sighash = SigHashCache::new(&tx).segwitv0_sighash(
                            index as usize,
                            &script,
                            value,
                            SigHashType::All,
                        );

                        let sig = SECP256K1.sign(&Message::from(sighash), &sk);

                        let mut serialized_signature = sig.serialize_der().to_vec();
                        serialized_signature.push(SigHashType::All as u8);

                        tx.input[index as usize].witness.script_witness =
                            vec![serialized_signature, pk.serialize().to_vec()];

                        Ok(tx)
                    }
                },
                // TODO: Do the same as in the loan transaction for the fee
                Amount::from_sat(10_000),
            )
            .await
            .unwrap();

        client
            .send_raw_transaction(&loan_repayment_transaction)
            .await
            .unwrap();
    }

    // #[tokio::test]
    // async fn lend_and_liquidate() {
    //     let tc_client = Cli::default();
    //     let (client, _container) = {
    //         let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

    //         (
    //             elements_harness::Client::new(blockchain.node_url.clone().into_string()).unwrap(),
    //             blockchain,
    //         )
    //     };

    //     let bitcoin_asset_id = client.get_bitcoin_asset_id().await.unwrap();
    //     let usdt_asset_id = client.issueasset(40.0, 0.0, false).await.unwrap().asset;

    //     // TODO: Use a separate wallet per actor. Using the same wallet is confusing and bug-prone.
    //     let (lender, _lender_address) = {
    //         let address = client
    //             .get_new_address(Some("blech32".to_string()))
    //             .await
    //             .unwrap();
    //         let principal_inputs =
    //             generate_input(&client, Amount::from_btc(2.0).unwrap(), usdt_asset_id)
    //                 .await
    //                 .unwrap();

    //         let lender = Lender0::new(
    //             bitcoin_asset_id,
    //             usdt_asset_id,
    //             principal_inputs,
    //             address.clone(),
    //         );

    //         (lender, address)
    //     };

    //     let tx_fee = Amount::from_sat(10_000);
    //     let (borrower, _borrower_address) = {
    //         let collateral_amount = Amount::ONE_BTC;
    //         let collateral_inputs =
    //             generate_input(&client, collateral_amount * 2, bitcoin_asset_id)
    //                 .await
    //                 .unwrap();
    //         let address = client
    //             .get_new_address(Some("blech32".to_string()))
    //             .await
    //             .unwrap();

    //         let timelock = 0;

    //         let borrower = Borrower0::new(
    //             address.clone(),
    //             collateral_amount,
    //             collateral_inputs,
    //             tx_fee,
    //             timelock,
    //             bitcoin_asset_id,
    //             usdt_asset_id,
    //         )
    //         .unwrap();

    //         (borrower, address)
    //     };

    //     let loan_request = borrower.loan_request();

    //     let lender = lender.interpret(loan_request);
    //     let loan_response = lender.loan_response();

    //     let borrower = borrower.interpret(loan_response).unwrap();
    //     let loan_transaction = borrower
    //         .sign({
    //             let client = client.clone();
    //             |transaction| async move { client.sign_raw_transaction(&transaction).await }
    //         })
    //         .await
    //         .unwrap();

    //     let loan_transaction = lender
    //         .finalise_loan(loan_transaction, {
    //             let client = client.clone();
    //             |transaction| async move { client.sign_raw_transaction(&transaction).await }
    //         })
    //         .await
    //         .unwrap();

    //     client
    //         .send_raw_transaction(&loan_transaction)
    //         .await
    //         .unwrap();

    //     let liquidation_transaction = lender.liquidation_transaction(tx_fee).unwrap();

    //     client
    //         .send_raw_transaction(&liquidation_transaction)
    //         .await
    //         .unwrap();
    // }

    async fn find_inputs(
        client: &elements_harness::Client,
        asset: AssetId,
        amount: Amount,
    ) -> Result<Vec<crate::Input>> {
        let inputs = client.select_inputs_for(asset, amount, false).await?;

        // let address = client
        //     .get_new_address(Some("blech32".into()))
        //     .await
        //     .unwrap();
        // let txid = client
        //     .send_asset_to_address(&address, amount, Some(asset))
        //     .await
        //     .unwrap();

        // // let inputs = client.select_inputs_for(asset, amount, false).await?;
        // let inputs = {
        //     let tx = client.get_raw_transaction(txid).await.unwrap();

        //     let vout = tx
        //         .output
        //         .iter()
        //         .position(|out| out.script_pubkey == address.script_pubkey())
        //         .unwrap();

        //     let outpoint = OutPoint {
        //         txid,
        //         vout: vout as u32,
        //     };
        //     let tx_out = tx.output[vout].clone();

        //     vec![(outpoint, tx_out)]
        // };

        let master_blinding_key = client.dumpmasterblindingkey().await?;
        let master_blinding_key = hex::decode(master_blinding_key)?;

        let inputs = inputs
            .into_iter()
            .map(|(outpoint, tx_out)| {
                let input_blinding_sk =
                    derive_blinding_key(master_blinding_key.clone(), tx_out.script_pubkey.clone())?;

                Result::<_, anyhow::Error>::Ok(crate::Input {
                    tx_in: TxIn {
                        previous_output: outpoint,
                        is_pegin: false,
                        has_issuance: false,
                        script_sig: Default::default(),
                        sequence: 0,
                        asset_issuance: Default::default(),
                        witness: Default::default(),
                    },
                    original_tx_out: tx_out,
                    blinding_key: input_blinding_sk,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(inputs)
    }

    fn derive_blinding_key(
        master_blinding_key: Vec<u8>,
        script_pubkey: Script,
    ) -> Result<SecretKey> {
        use hmac::{Hmac, Mac, NewMac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_varkey(&master_blinding_key)
            .expect("HMAC can take key of any size");
        mac.update(script_pubkey.as_bytes());

        let result = mac.finalize();
        let blinding_sk = SecretKey::from_slice(&result.into_bytes())?;

        Ok(blinding_sk)
    }

    // pub fn sign_with_key<C>(
    //     secp: &Secp256k1<C>,
    //     cache: &mut SigHashCache<&Transaction>,
    //     index: usize,
    //     input_sk: &SecretKey,
    //     value: confidential::Value,
    // ) -> Vec<Vec<u8>>
    // where
    //     C: Signing,
    // {
    //     let input_pk = PublicKey::from_secret_key(&secp, &input_sk);

    //     let hash = hash160::Hash::hash(&input_pk.serialize());
    //     let script = Builder::new()
    //         .push_opcode(opcodes::all::OP_DUP)
    //         .push_opcode(opcodes::all::OP_HASH160)
    //         .push_slice(&hash.into_inner())
    //         .push_opcode(opcodes::all::OP_EQUALVERIFY)
    //         .push_opcode(opcodes::all::OP_CHECKSIG)
    //         .into_script();

    //     let sighash = cache.segwitv0_sighash(index, &script, value, SigHashType::All);

    //     let sig = secp.sign(&Message::from(sighash), &input_sk);

    //     let mut serialized_signature = sig.serialize_der().to_vec();
    //     serialized_signature.push(SigHashType::All as u8);

    //     vec![serialized_signature, input_pk.serialize().to_vec()]
    // }
}
