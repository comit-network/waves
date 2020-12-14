use anyhow::Result;
use bdk::{
    bitcoin::Amount,
    database::{BatchOperations, Database},
    wallet::coin_selection::CoinSelectionResult,
    wallet::coin_selection::{BranchAndBoundCoinSelection, CoinSelectionAlgorithm},
};
use elements_fun::{ExplicitTxOut, OutPoint};

#[derive(Clone, PartialEq)]
pub struct Utxo {
    outpoint: OutPoint,
    txout: ExplicitTxOut,
}

impl From<Utxo> for bdk::UTXO {
    fn from(utxo: Utxo) -> Self {
        let value = utxo.txout.value.0;
        let script_pubkey = utxo.txout.script_pubkey.into_bytes();
        let script_pubkey = bdk::bitcoin::Script::from(script_pubkey);

        Self {
            outpoint: bdk::bitcoin::OutPoint {
                txid: bdk::bitcoin::Txid::from_hash(utxo.outpoint.txid.as_hash()),
                vout: utxo.outpoint.vout,
            },
            txout: bdk::bitcoin::TxOut {
                value,
                script_pubkey,
            },
            script_type: bdk::ScriptType::External,
        }
    }
}

pub fn coin_select(utxos: Vec<Utxo>, target: Amount) -> Result<Vec<Utxo>> {
    let algorithm = BranchAndBoundCoinSelection::default();

    let bdk_utxos = utxos
        .iter()
        .cloned()
        .map(bdk::UTXO::from)
        .map(|utxo| (utxo, 0))
        .collect();

    let CoinSelectionResult {
        selected: selected_utxos,
        ..
    } = algorithm.coin_select(
        &DummyDb,
        Vec::new(),
        bdk_utxos,
        bdk::FeeRate::default(),
        target.as_sat(),
        0.0,
    )?;

    let selected_utxos = selected_utxos
        .iter()
        .map(|bdk_utxo| {
            utxos
                .iter()
                .find(|utxo| {
                    bdk_utxo.outpoint.txid.as_hash() == utxo.outpoint.txid.as_hash()
                        && bdk_utxo.outpoint.vout == utxo.outpoint.vout
                })
                .expect("same source of utxos")
        })
        .cloned()
        .collect();

    Ok(selected_utxos)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use elements_fun::{AssetId, ExplicitAsset, ExplicitValue, Script, Txid};

    use super::*;

    #[test]
    fn trivial_coin_selection() {
        let utxo = Utxo {
            outpoint: OutPoint {
                txid: Txid::default(),
                vout: 0,
            },
            txout: ExplicitTxOut {
                asset: ExplicitAsset(
                    AssetId::from_str(
                        "0000000000000000000000000000000000000000000000000000000000000000",
                    )
                    .unwrap(),
                ),
                value: ExplicitValue(100_000_000),
                script_pubkey: Script::new(),
                nonce: None,
            },
        };

        let selection = coin_select(vec![utxo.clone()], Amount::from_sat(90_000_000)).unwrap();

        assert!(selection.len() == 1);
        assert!(selection.contains(&utxo));
    }
}

/// A placeholder for the `database` argument required by
/// `CoinSelectionAlgorithm::coin_select`, but which is never actually
/// used in the trait implementation.
struct DummyDb;

impl Database for DummyDb {
    fn check_descriptor_checksum<B: AsRef<[u8]>>(
        &mut self,
        _script_type: bdk::ScriptType,
        _bytes: B,
    ) -> Result<(), bdk::Error> {
        todo!()
    }

    fn iter_script_pubkeys(
        &self,
        _script_type: Option<bdk::ScriptType>,
    ) -> Result<Vec<bdk::bitcoin::Script>, bdk::Error> {
        todo!()
    }

    fn iter_utxos(&self) -> Result<Vec<bdk::UTXO>, bdk::Error> {
        todo!()
    }

    fn iter_raw_txs(&self) -> Result<Vec<bdk::bitcoin::Transaction>, bdk::Error> {
        todo!()
    }

    fn iter_txs(&self, _include_raw: bool) -> Result<Vec<bdk::TransactionDetails>, bdk::Error> {
        todo!()
    }

    fn get_script_pubkey_from_path(
        &self,
        _script_type: bdk::ScriptType,
        _child: u32,
    ) -> Result<Option<bdk::bitcoin::Script>, bdk::Error> {
        todo!()
    }

    fn get_path_from_script_pubkey(
        &self,
        _script: &bdk::bitcoin::Script,
    ) -> Result<Option<(bdk::ScriptType, u32)>, bdk::Error> {
        todo!()
    }

    fn get_utxo(
        &self,
        _outpoint: &bdk::bitcoin::OutPoint,
    ) -> Result<Option<bdk::UTXO>, bdk::Error> {
        todo!()
    }

    fn get_raw_tx(
        &self,
        _txid: &bdk::bitcoin::Txid,
    ) -> Result<Option<bdk::bitcoin::Transaction>, bdk::Error> {
        todo!()
    }

    fn get_tx(
        &self,
        _txid: &bdk::bitcoin::Txid,
        _include_raw: bool,
    ) -> Result<Option<bdk::TransactionDetails>, bdk::Error> {
        todo!()
    }

    fn get_last_index(&self, _script_type: bdk::ScriptType) -> Result<Option<u32>, bdk::Error> {
        todo!()
    }

    fn increment_last_index(&mut self, _script_type: bdk::ScriptType) -> Result<u32, bdk::Error> {
        todo!()
    }
}

impl BatchOperations for DummyDb {
    fn set_script_pubkey(
        &mut self,
        _script: &bdk::bitcoin::Script,
        _script_type: bdk::ScriptType,
        _child: u32,
    ) -> Result<(), bdk::Error> {
        todo!()
    }

    fn set_utxo(&mut self, _utxo: &bdk::UTXO) -> Result<(), bdk::Error> {
        todo!()
    }

    fn set_raw_tx(&mut self, _transaction: &bdk::bitcoin::Transaction) -> Result<(), bdk::Error> {
        todo!()
    }

    fn set_tx(&mut self, _transaction: &bdk::TransactionDetails) -> Result<(), bdk::Error> {
        todo!()
    }

    fn set_last_index(
        &mut self,
        _script_type: bdk::ScriptType,
        _value: u32,
    ) -> Result<(), bdk::Error> {
        todo!()
    }

    fn del_script_pubkey_from_path(
        &mut self,
        _script_type: bdk::ScriptType,
        _child: u32,
    ) -> Result<Option<bdk::bitcoin::Script>, bdk::Error> {
        todo!()
    }

    fn del_path_from_script_pubkey(
        &mut self,
        _script: &bdk::bitcoin::Script,
    ) -> Result<Option<(bdk::ScriptType, u32)>, bdk::Error> {
        todo!()
    }

    fn del_utxo(
        &mut self,
        _outpoint: &bdk::bitcoin::OutPoint,
    ) -> Result<Option<bdk::UTXO>, bdk::Error> {
        todo!()
    }

    fn del_raw_tx(
        &mut self,
        _txid: &bdk::bitcoin::Txid,
    ) -> Result<Option<bdk::bitcoin::Transaction>, bdk::Error> {
        todo!()
    }

    fn del_tx(
        &mut self,
        _txid: &bdk::bitcoin::Txid,
        _include_raw: bool,
    ) -> Result<Option<bdk::TransactionDetails>, bdk::Error> {
        todo!()
    }

    fn del_last_index(&mut self, _script_type: bdk::ScriptType) -> Result<Option<u32>, bdk::Error> {
        todo!()
    }
}
