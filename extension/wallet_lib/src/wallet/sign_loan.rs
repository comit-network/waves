use covenants::Borrower1;
use elements::{secp256k1_zkp::SECP256K1, sighash::SigHashCache, Transaction};
use futures::lock::Mutex;
use swap::sign_with_key;

use crate::{
    storage::Storage,
    wallet::{current, get_txouts},
    Wallet,
};

pub async fn sign_loan(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
) -> Result<Transaction, Error> {
    let storage = Storage::local_storage().map_err(|_| Error::LoadState)?;
    let borrower = storage
        .get_item::<String>("borrower_state")
        .map_err(|_| Error::LoadState)?
        .ok_or(Error::LoadState)?;
    let borrower: Borrower1 =
        serde_json::from_str(&borrower).map_err(|_| Error::DeserializeState)?;

    let wallet = current(&name, current_wallet)
        .await
        .map_err(|_| Error::LoadWallet)?;

    let txouts = get_txouts(&wallet, |utxo, txout| Ok(Some((utxo, txout))))
        .await
        .map_err(|e| Error::GetTxOuts(format!("{:#}", e)))?;

    let loan_transaction = borrower
        .sign(|mut transaction| async {
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
        })
        .await
        .map_err(|_| Error::Sign)?;

    Ok(loan_transaction)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Wallet is not loaded")]
    LoadWallet,
    #[error("Failed to load borrower state")]
    LoadState,
    #[error("Failed to deserialize borrower state")]
    DeserializeState,
    #[error("Failed to get transaction outputs: {0}")]
    GetTxOuts(String),
    #[error("Failed to sign transaction")]
    Sign,
    #[error("Failed to broadcast transaction: {0}")]
    Send(String),
}
