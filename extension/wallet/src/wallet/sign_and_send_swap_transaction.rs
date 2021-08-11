use crate::{
    wallet::{current, get_txouts, Wallet},
    ESPLORA_CLIENT,
};
use anyhow::Result;
use baru::swap::{alice_finalize_transaction, sign_with_key};
use elements::{secp256k1_zkp::SECP256K1, sighash::SigHashCache, Transaction, Txid};
use futures::lock::Mutex;
use wasm_bindgen::UnwrapThrowExt;

pub(crate) async fn sign_and_send_swap_transaction(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    transaction: Transaction,
) -> Result<Txid, Error> {
    let client = ESPLORA_CLIENT.lock().expect_throw("can get lock");

    let wallet = current(&name, current_wallet)
        .await
        .map_err(Error::LoadWallet)?;

    let txouts = get_txouts(&wallet, |utxo, txout| Ok(Some((utxo, txout))))
        .await
        .map_err(Error::GetTxOuts)?;

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
    .map_err(Error::Sign)?;

    let txid = client.broadcast(transaction).await.map_err(Error::Send)?;

    Ok(txid)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Wallet is not loaded: {0}")]
    LoadWallet(anyhow::Error),
    #[error("Failed to get transaction outputs: {0}")]
    GetTxOuts(anyhow::Error),
    #[error("Failed to sign transaction: {0}")]
    Sign(anyhow::Error),
    #[error("Failed to broadcast transaction: {0}")]
    Send(anyhow::Error),
}
