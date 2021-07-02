use crate::{
    esplora::broadcast,
    wallet::{current, get_txouts, Wallet},
};
use anyhow::{Context, Result};
use elements::{encode::deserialize, secp256k1_zkp::SECP256K1, sighash::SigHashCache, Txid};
use futures::lock::Mutex;
use swap::{alice_finalize_transaction, sign_with_key};

pub async fn sign_and_send_swap_transaction(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    tx_hex: String,
) -> Result<Txid, Error> {
    let transaction = deserialize(&hex::decode(&tx_hex).context("failed to decode string as hex")?)
        .context("failed to deserialize bytes as elements transaction")?;

    let wallet = current(&name, current_wallet)
        .await
        .map_err(|_| Error::LoadWallet)?;

    let txouts = get_txouts(&wallet, |utxo, txout| Ok(Some((utxo, txout))))
        .await
        .map_err(|e| Error::GetTxOuts(format!("{:#}", e)))?;

    let transaction = alice_finalize_transaction(transaction, |mut transaction| async {
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

    let txid = broadcast(transaction)
        .await
        .map_err(|e| Error::Send(format!("{:#}", e)))?;

    Ok(txid)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Wallet is not loaded")]
    LoadWallet,
    #[error("Failed to get transaction outputs: {0}")]
    GetTxOuts(String),
    #[error("Failed to sign transaction")]
    Sign,
    #[error("Failed to broadcast transaction: {0}")]
    Send(String),
    #[error("Unclassified error: {0}")]
    Other(#[from] anyhow::Error),
}
