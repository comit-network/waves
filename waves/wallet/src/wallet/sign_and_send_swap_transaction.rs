use crate::{
    esplora::broadcast,
    wallet::{current, get_txouts, Wallet},
};
use anyhow::{Context, Result};
use elements_fun::{encode::deserialize, secp256k1::SECP256K1, sighash::SigHashCache, Txid};
use futures::lock::Mutex;
use swap::{alice_finalize_transaction, sign_with_key};

pub async fn sign_and_send_swap_transaction(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    transaction: String,
) -> Result<Txid> {
    let transaction =
        deserialize(&hex::decode(&transaction).context("failed to decode string as hex")?)
            .context("failed to deserialize bytes as elements transaction")?;

    let wallet = current(&name, current_wallet).await?;

    let txouts = get_txouts(&wallet, |utxo, txout| Ok(Some((utxo, txout)))).await?;

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
                    output.as_confidential().unwrap().value,
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
    .context("failed to finalize transaction as alice")?;

    let txid = broadcast(transaction)
        .await
        .context("failed to broadcast transaction")?;

    Ok(txid)
}
