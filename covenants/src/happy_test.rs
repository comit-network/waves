#[cfg(test)]
mod tests {
    use crate::{Borrower0, Lender0};
    use anyhow::{Context, Result};
    use elements::bitcoin::Amount;
    use elements::secp256k1::SecretKey;
    use elements::secp256k1::SECP256K1;
    use elements::AssetId;
    use elements::AssetIssuance;
    use elements::ExplicitTxOut;
    use elements::{OutPoint, Script, TxIn, TxInWitness};
    use elements_harness::{elementd_rpc::ElementsRpc, Elementsd};
    use testcontainers::clients::Cli;

    #[tokio::test]
    async fn loan_protocol() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (
                elements_harness::Client::new(blockchain.node_url.clone().into_string()).unwrap(),
                blockchain,
            )
        };

        let bitcoin_asset_id = client.get_bitcoin_asset_id().await.unwrap();
        let usdt_asset_id = client.issueasset(40.0, 0.0, false).await.unwrap().asset;

        // TODO: Use a separate wallet per actor. Using the same wallet is confusing and bug-prone.
        let (lender, _lender_address) = {
            let address = client
                .get_new_address(Some("bech32".to_string()))
                .await
                .unwrap();
            let principal_inputs =
                generate_unblinded_input(&client, Amount::from_btc(2.0).unwrap(), usdt_asset_id)
                    .await
                    .unwrap();

            let lender = Lender0::new(
                bitcoin_asset_id,
                usdt_asset_id,
                principal_inputs,
                address.clone(),
            );

            (lender, address)
        };

        let tx_fee = Amount::from_sat(10_000);
        let (borrower, _borrower_address) = {
            let collateral_amount = Amount::ONE_BTC;
            let collateral_inputs =
                generate_unblinded_input(&client, collateral_amount * 2, bitcoin_asset_id)
                    .await
                    .unwrap();
            let address = client
                .get_new_address(Some("bech32".to_string()))
                .await
                .unwrap();

            let timelock = 0;

            let borrower = Borrower0::new(
                address.clone(),
                collateral_amount,
                collateral_inputs,
                tx_fee,
                timelock,
                bitcoin_asset_id,
                usdt_asset_id,
            )
            .unwrap();

            (borrower, address)
        };

        let loan_request = borrower.loan_request();

        let lender = lender.interpret(loan_request);
        let loan_response = lender.loan_response();

        let borrower = borrower.interpret(loan_response).unwrap();
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

        let loan_repayment_transaction =
            borrower
                .loan_repayment_transaction(
                    {
                        let client = client.clone();
                        |amount, asset| async move {
                            generate_unblinded_input(&client, amount, asset).await
                        }
                    },
                    {
                        let client = client.clone();
                        |transaction| async move { client.sign_raw_transaction(&transaction).await }
                    },
                    tx_fee,
                )
                .await
                .unwrap();

        client
            .send_raw_transaction(&loan_repayment_transaction)
            .await
            .unwrap();
    }

    async fn generate_unblinded_input(
        client: &elements_harness::Client,
        amount: Amount,
        asset: AssetId,
    ) -> Result<Vec<crate::Input>> {
        let address = client.get_new_address(Some("bech32".to_string())).await?;
        let address = client.getaddressinfo(&address).await?.unconfidential;
        let txid = client
            .send_asset_to_address(&address, amount, Some(asset))
            .await?;
        let tx = client.get_raw_transaction(txid).await?;

        let vout = tx
            .output
            .iter()
            .position(|out| {
                out.asset.is_explicit()
                    && out.asset.explicit().unwrap() == asset
                    && out.value.is_explicit()
                    && out.value.explicit().unwrap() == amount.as_sat()
            })
            .with_context(|| {
                format!(
                    "no explicit output with asset {} and amount {}",
                    asset, amount,
                )
            })?;

        Ok(vec![crate::Input {
            amount,
            tx_in: TxIn {
                previous_output: OutPoint {
                    txid,
                    vout: vout as u32,
                },
                is_pegin: false,
                has_issuance: false,
                script_sig: Script::new(),
                sequence: 0,
                asset_issuance: AssetIssuance::default(),
                witness: TxInWitness::default(),
            },
        }])
    }

    // TODO: Using this function to select inputs instead of
    // `generate_unblinded_input()` seems better, but fails
    async fn _find_inputs(
        client: &elements_harness::Client,
        asset: AssetId,
        amount: Amount,
    ) -> Result<Vec<crate::Input>> {
        let inputs = client.select_inputs_for(asset, amount, false).await?;
        let master_blinding_key = client.dumpmasterblindingkey().await?;
        let master_blinding_key = hex::decode(master_blinding_key)?;

        let inputs = inputs
            .iter()
            .filter_map(|(outpoint, tx_out)| {
                use hmac::{Hmac, Mac, NewMac};
                use sha2::Sha256;

                let mut mac = Hmac::<Sha256>::new_varkey(&master_blinding_key)
                    .expect("HMAC can take key of any size");
                mac.update(tx_out.script_pubkey.as_bytes());

                let result = mac.finalize();
                let blinding_sk = SecretKey::from_slice(&result.into_bytes()).expect("valid sk");

                let amount = match (tx_out.to_explicit(), tx_out.to_confidential()) {
                    (Some(ExplicitTxOut { value, .. }), None) => value,
                    (None, Some(conf)) => conf.unblind(SECP256K1, blinding_sk).unwrap().value,
                    _ => return None,
                };

                Some(crate::Input {
                    amount: Amount::from_sat(amount),
                    tx_in: TxIn {
                        previous_output: *outpoint,
                        is_pegin: false,
                        has_issuance: false,
                        script_sig: Script::new(),
                        sequence: 0,
                        asset_issuance: AssetIssuance::default(),
                        witness: TxInWitness::default(),
                    },
                })
            })
            .collect();

        Ok(inputs)
    }
}
