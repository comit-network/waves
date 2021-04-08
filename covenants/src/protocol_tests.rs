#[cfg(test)]
mod tests {
    use crate::{make_keypair, Borrower0, Lender0};
    use anyhow::{anyhow, Context, Result};
    use bitcoin_hashes::Hash;
    use elements::{
        bitcoin::{util::psbt::serialize::Serialize, Amount, PublicKey},
        opcodes,
        script::Builder,
        secp256k1::{rand::thread_rng, SecretKey, SECP256K1},
        sighash::SigHashCache,
        Address, AddressParams, AssetId, OutPoint, Script, SigHashType, Transaction, TxIn, TxOut,
        Txid,
    };
    use elements_harness::{elementd_rpc::ElementsRpc, Elementsd};
    use testcontainers::clients::Cli;

    #[tokio::test]
    async fn borrow_and_repay() {
        init_logger();

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

        let miner_address = client.get_new_segwit_confidential_address().await.unwrap();

        client
            .send_asset_to_address(&miner_address, Amount::from_btc(5.0).unwrap(), None)
            .await
            .unwrap();
        client.generatetoaddress(10, &miner_address).await.unwrap();

        let (borrower, borrower_wallet) = {
            let mut wallet = Wallet::new();

            let collateral_amount = Amount::ONE_BTC;

            let address = wallet.address();
            let address_blinding_sk = wallet.dump_blinding_sk();

            // fund borrower address with bitcoin
            let txid = client
                .send_asset_to_address(&address, collateral_amount * 2, Some(bitcoin_asset_id))
                .await
                .unwrap();

            wallet.add_known_utxo(&client, txid).await;

            // fund wallet with some usdt to pay back the loan later on
            let txid = client
                .send_asset_to_address(
                    &address,
                    Amount::from_btc(2.0).unwrap(),
                    Some(usdt_asset_id),
                )
                .await
                .unwrap();
            wallet.add_known_utxo(&client, txid).await;

            client.generatetoaddress(1, &miner_address).await.unwrap();

            let collateral_inputs = wallet
                .find_inputs(bitcoin_asset_id, collateral_amount * 2)
                .await
                .unwrap();

            let timelock = 10;

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

            (borrower, wallet)
        };

        let (lender, _lender_address) = {
            let address = client.get_new_segwit_confidential_address().await.unwrap();

            let lender = Lender0::new(bitcoin_asset_id, usdt_asset_id, address.clone()).unwrap();

            (lender, address)
        };

        let loan_request = borrower.loan_request();

        let lender = lender
            .interpret(
                &mut thread_rng(),
                &SECP256K1,
                {
                    let client = client.clone();
                    |amount, asset| async move { find_inputs(&client, asset, amount).await }
                },
                loan_request,
            )
            .await
            .unwrap();
        let loan_response = lender.loan_response();

        let borrower = borrower.interpret(&SECP256K1, loan_response).unwrap();
        let loan_transaction = borrower
            .sign({
                let wallet = borrower_wallet.clone();
                |transaction| async move { Ok(wallet.sign_all_inputs(transaction)) }
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
                    let borrower_wallet = borrower_wallet.clone();
                    |amount, asset| async move { borrower_wallet.find_inputs(asset, amount).await }
                },
                |tx| async move { Ok(borrower_wallet.sign_all_inputs(tx)) },
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

    fn init_logger() {
        // force enabling log output
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn lend_and_liquidate() {
        init_logger();

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

        let miner_address = client.get_new_segwit_confidential_address().await.unwrap();
        client
            .send_asset_to_address(&miner_address, Amount::from_btc(5.0).unwrap(), None)
            .await
            .unwrap();
        client.generatetoaddress(10, &miner_address).await.unwrap();

        let (borrower, borrower_wallet) = {
            let mut wallet = Wallet::new();

            let collateral_amount = Amount::ONE_BTC;

            let address = wallet.address();
            let address_blinding_sk = wallet.dump_blinding_sk();

            // fund borrower address with bitcoin
            let txid = client
                .send_asset_to_address(&address, collateral_amount * 2, Some(bitcoin_asset_id))
                .await
                .unwrap();

            wallet.add_known_utxo(&client, txid).await;

            // fund wallet with some usdt to pay back the loan later on
            let txid = client
                .send_asset_to_address(
                    &address,
                    Amount::from_btc(2.0).unwrap(),
                    Some(usdt_asset_id),
                )
                .await
                .unwrap();
            wallet.add_known_utxo(&client, txid).await;

            client.generatetoaddress(1, &miner_address).await.unwrap();

            let collateral_inputs = wallet
                .find_inputs(bitcoin_asset_id, collateral_amount * 2)
                .await
                .unwrap();

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

            (borrower, wallet)
        };

        let (lender, _lender_address) = {
            let address = client.get_new_segwit_confidential_address().await.unwrap();

            let lender = Lender0::new(bitcoin_asset_id, usdt_asset_id, address.clone()).unwrap();

            (lender, address)
        };

        let loan_request = borrower.loan_request();

        let lender = lender
            .interpret(
                &mut thread_rng(),
                &SECP256K1,
                {
                    let client = client.clone();
                    |amount, asset| async move { find_inputs(&client, asset, amount).await }
                },
                loan_request,
            )
            .await
            .unwrap();
        let loan_response = lender.loan_response();

        let borrower = borrower.interpret(&SECP256K1, loan_response).unwrap();
        let loan_transaction = borrower
            .sign(|transaction| async move { Ok(borrower_wallet.sign_all_inputs(transaction)) })
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

        let liquidation_transaction = lender
            .liquidation_transaction(&mut thread_rng(), &SECP256K1, Amount::from_sat(10_000))
            .unwrap();

        client
            .send_raw_transaction(&liquidation_transaction)
            .await
            .unwrap();
    }

    async fn find_inputs(
        client: &elements_harness::Client,
        asset: AssetId,
        amount: Amount,
    ) -> Result<Vec<crate::Input>> {
        let inputs = client.select_inputs_for(asset, amount, false).await?;

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

    #[derive(Clone)]
    pub struct Wallet {
        keypair: (SecretKey, PublicKey),
        blinder_keypair: (SecretKey, PublicKey),
        address: Address,
        known_utxos: Vec<(Txid, usize, TxOut)>,
    }

    impl Wallet {
        pub fn new() -> Self {
            let (sk, pk) = make_keypair();
            let (blinder_sk, blinder_pk) = make_keypair();

            let address = Address::p2wpkh(&pk, Some(blinder_pk.key), &AddressParams::ELEMENTS);

            Wallet {
                keypair: (sk, pk),
                blinder_keypair: (blinder_sk, blinder_pk),
                address,
                known_utxos: vec![],
            }
        }

        pub fn address(&self) -> Address {
            self.address.clone()
        }

        pub fn dump_blinding_sk(&self) -> SecretKey {
            self.blinder_keypair.0
        }

        pub async fn add_known_utxo(&mut self, client: &elements_harness::Client, txid: Txid) {
            let transaction = client.get_raw_transaction(txid).await.unwrap();

            let maybe_vout = transaction
                .output
                .iter()
                .position(|txout| txout.script_pubkey == self.address.script_pubkey());

            if let Some(vout) = maybe_vout {
                let txout = transaction.output.get(vout).unwrap();
                self.known_utxos.push((txid, vout, txout.clone()));
            }
        }

        // TODO use amounts to assert that we have enough funding
        async fn find_inputs(
            &self,
            want_asset: AssetId,
            _amount: Amount,
        ) -> Result<Vec<crate::Input>> {
            let utxos = self.known_utxos.clone();

            let inputs = utxos
                .into_iter()
                .filter_map(|(txid, vout, original_tx_out)| {
                    let found_asset = if let Some(confidential_tx_out) =
                        original_tx_out.clone().into_confidential()
                    {
                        let out = confidential_tx_out
                            .unblind(SECP256K1, self.blinder_keypair.0)
                            .context("could not unblind output")
                            .unwrap();
                        out.asset
                    } else {
                        original_tx_out
                            .asset
                            .explicit()
                            .ok_or_else(|| anyhow!("Should be explicit"))
                            .unwrap()
                    };
                    if found_asset != want_asset {
                        log::debug!(
                            "Found transaction with different asset: found: {}, wanted: {}",
                            found_asset,
                            want_asset
                        );
                        return None;
                    }

                    Some(crate::Input {
                        tx_in: TxIn {
                            previous_output: OutPoint {
                                txid,
                                vout: vout as u32,
                            },
                            is_pegin: false,
                            has_issuance: false,
                            script_sig: Default::default(),
                            sequence: 0,
                            asset_issuance: Default::default(),
                            witness: Default::default(),
                        },
                        original_tx_out,
                        blinding_key: self.blinder_keypair.0,
                    })
                })
                .collect::<Vec<_>>();
            Ok(inputs)
        }

        fn sign_all_inputs(&self, tx: Transaction) -> Transaction {
            let mut tx_to_sign = tx;
            // first try to find out which utxos we know
            let known_inputs = tx_to_sign.clone().input.into_iter().filter_map(|txin| {
                if let Some((_, _, outpoint)) = self
                    .known_utxos
                    .iter()
                    .find(|(txid, _, _)| txid == &txin.previous_output.txid)
                {
                    Some((txin, outpoint.value))
                } else {
                    None
                }
            });

            known_inputs.into_iter().for_each(|(txin, value)| {
                let hash = bitcoin_hashes::hash160::Hash::hash(&self.keypair.1.serialize());
                let script = Builder::new()
                    .push_opcode(opcodes::all::OP_DUP)
                    .push_opcode(opcodes::all::OP_HASH160)
                    .push_slice(&hash.into_inner())
                    .push_opcode(opcodes::all::OP_EQUALVERIFY)
                    .push_opcode(opcodes::all::OP_CHECKSIG)
                    .into_script();

                let index = tx_to_sign
                    .input
                    .iter()
                    .position(|other| other == &txin)
                    .unwrap();

                let sighash = SigHashCache::new(&tx_to_sign).segwitv0_sighash(
                    index,
                    &script,
                    value,
                    SigHashType::All,
                );
                let sig = SECP256K1.sign(&secp256k1_zkp::Message::from(sighash), &self.keypair.0);

                let mut serialized_signature = sig.serialize_der().to_vec();
                serialized_signature.push(SigHashType::All as u8);

                tx_to_sign.input[index as usize].witness.script_witness =
                    vec![serialized_signature, self.keypair.1.serialize().to_vec()];
            });

            tx_to_sign
        }
    }
}
