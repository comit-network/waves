use crate::{
    esplora::{broadcast, fetch_transaction},
    storage::Storage,
    wallet::{current, get_txouts, LoanDetails},
    Wallet, DEFAULT_SAT_PER_VBYTE,
};
use anyhow::{bail, Context, Result};
use baru::{input::Input, loan::Borrower1, swap::sign_with_key};
use coin_selection::coin_select;
use elements::{
    bitcoin::util::amount::Amount, secp256k1_zkp::SECP256K1, sighash::SigHashCache, OutPoint, Txid,
};
use futures::lock::Mutex;
use rand::thread_rng;

// TODO: Parts of the implementation are very similar to what we do in
// `sign_and_send_swap_transaction`. We could extract common
// functionality into crate-local functions
pub async fn repay_loan(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    loan_txid: Txid,
) -> Result<Txid> {
    // TODO: Only abort early if this fails because the transaction
    // hasn't been mined
    if fetch_transaction(loan_txid).await.is_err() {
        bail!("No loan with txid {}", loan_txid)
    }

    let storage = Storage::local_storage()?;

    let borrower = storage
        .get_item::<String>(&format!("loan_state:{}", loan_txid))?
        .context("Unable to find state for loan")?;
    let borrower = serde_json::from_str::<Borrower1>(&borrower)
        .context("Failed to deserialize `Borrower1`")?;

    let blinding_key = {
        let wallet = current(&name, current_wallet).await?;
        wallet.blinding_key()
    };

    let coin_selector = {
        let name = name.clone();
        |amount, asset| async move {
            let wallet = current(&name, current_wallet).await?;

            let utxos = get_txouts(&wallet, |utxo, txout| {
                Ok({
                    let unblinded_txout = txout.unblind(SECP256K1, blinding_key)?;
                    let outpoint = OutPoint {
                        txid: utxo.txid,
                        vout: utxo.vout,
                    };
                    let candidate_asset = unblinded_txout.asset;

                    if candidate_asset == asset {
                        Some((
                            coin_selection::Utxo {
                                outpoint,
                                value: unblinded_txout.value,
                                script_pubkey: txout.script_pubkey.clone(),
                                asset: candidate_asset,
                            },
                            txout,
                        ))
                    } else {
                        log::debug!(
                            "utxo {} with asset id {} is not the target asset, ignoring",
                            outpoint,
                            candidate_asset
                        );
                        None
                    }
                })
            })
            .await?;

            // We are selecting coins with an asset which cannot be
            // used to pay for fees
            let zero_fee_rate = 0f32;
            let zero_fee_offset = Amount::ZERO;

            let output = coin_select(
                utxos.iter().map(|(utxo, _)| utxo).cloned().collect(),
                amount,
                zero_fee_rate,
                zero_fee_offset,
            )?;
            let selection = output
                .coins
                .iter()
                .map(|coin| {
                    let original_txout = utxos
                        .iter()
                        .find_map(|(utxo, txout)| (utxo.outpoint == coin.outpoint).then(|| txout))
                        .expect("same source of utxos")
                        .clone();

                    Input {
                        txin: coin.outpoint,
                        original_txout,
                        blinding_key,
                    }
                })
                .collect();

            Ok(selection)
        }
    };

    let signer = |mut transaction| async {
        let wallet = current(&name, current_wallet).await?;
        let txouts = get_txouts(&wallet, |utxo, txout| Ok(Some((utxo, txout)))).await?;

        let mut cache = SigHashCache::new(&transaction);

        let witnesses = transaction
            .clone()
            .input
            .iter()
            .enumerate()
            .filter_map(|(index, input)| {
                txouts
                    .iter()
                    .find(|(utxo, _)| {
                        utxo.txid == input.previous_output.txid
                            && utxo.vout == input.previous_output.vout
                    })
                    .map(|(_, txout)| (index, txout))
            })
            .map(|(index, output)| {
                // TODO: It is convenient to use this import, but
                // it is weird to use an API from the swap library
                // here. Maybe we should move it to a common
                // place, so it can be used for different
                // protocols
                let script_witness = sign_with_key(
                    SECP256K1,
                    &mut cache,
                    index,
                    &wallet.secret_key,
                    output.value,
                );

                (index, script_witness)
            })
            .collect::<Vec<_>>();

        for (index, witness) in witnesses {
            transaction.input[index].witness.script_witness = witness
        }

        Ok(transaction)
    };

    let loan_repayment_tx = borrower
        .loan_repayment_transaction(
            &mut thread_rng(),
            SECP256K1,
            coin_selector,
            signer,
            Amount::from_sat(DEFAULT_SAT_PER_VBYTE),
        )
        .await
        .context("Failed to build repayment transaction")?;

    let repayment_txid = broadcast(loan_repayment_tx)
        .await
        .context("Failed to send transaction")?;

    // TODO: Make sure that we can safely forget this i.e. sufficient
    // confirmations
    storage.remove_item(&format!("loan_state:{}", loan_txid))?;

    let open_loans = match storage.get_item::<String>("open_loans")? {
        Some(open_loans) => {
            serde_json::from_str(&open_loans).context("Failed to deserialize open loans")?
        }
        None => Vec::<LoanDetails>::new(),
    };

    let open_loans = open_loans
        .iter()
        .filter(|details| loan_txid != details.txid)
        .collect::<Vec<_>>();
    storage.set_item(
        "open_loans",
        serde_json::to_string(&open_loans).context("Failed to serialize open loans")?,
    )?;

    Ok(repayment_txid)
}
