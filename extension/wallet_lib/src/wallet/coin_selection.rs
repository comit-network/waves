#![allow(dead_code)]

use crate::wallet::avg_vbytes;
use anyhow::{anyhow, bail, Result};
use bdk::{
    bitcoin::{Amount, Denomination},
    database::{BatchOperations, Database},
    wallet::coin_selection::{
        BranchAndBoundCoinSelection, CoinSelectionAlgorithm, CoinSelectionResult,
    },
};
use elements::{AssetId, OutPoint, Script};

/// Select a subset of `utxos` to cover the `target` amount.
///
/// It makes use of a Branch and Bound coin selection algorithm
/// provided by `bdk`.
///
/// Only supports P2PK, P2PKH and P2WPKH UTXOs.
pub fn coin_select(
    utxos: Vec<Utxo>,
    target: Amount,
    fee_rate_sat_per_vbyte: f32,
    fee_offset: Amount,
) -> Result<Output> {
    let asset = utxos
        .first()
        .map(|utxo| utxo.asset)
        .ok_or_else(|| anyhow!("cannot select from empty utxo set"))?;

    if utxos.iter().any(|utxo| utxo.asset != asset) {
        bail!("all UTXOs must have the same asset ID")
    }

    let bdk_utxos = utxos
        .iter()
        .cloned()
        .filter_map(|utxo| {
            max_satisfaction_weight(&utxo.script_pubkey).map(|weight| (utxo, weight))
        })
        .map(|(utxo, weight)| (bdk::UTXO::from(utxo), weight))
        .collect();

    // a change is a regular output
    let size_of_change = avg_vbytes::OUTPUT;

    let CoinSelectionResult {
        selected: selected_utxos,
        fee_amount,
        ..
    } = BranchAndBoundCoinSelection::new(size_of_change).coin_select(
        &DummyDb,
        Vec::new(),
        bdk_utxos,
        bdk::FeeRate::from_sat_per_vb(fee_rate_sat_per_vbyte),
        target.as_sat(),
        fee_offset.as_sat() as f32,
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

    let recommended_fee = Amount::from_float_in(fee_amount.into(), Denomination::Satoshi)?;

    Ok(Output {
        coins: selected_utxos,
        target_amount: target,
        recommended_fee,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub value: u64,
    pub script_pubkey: Script,
    pub asset: AssetId,
}

impl From<Utxo> for bdk::UTXO {
    fn from(utxo: Utxo) -> Self {
        let value = utxo.value;
        let script_pubkey = utxo.script_pubkey.into_bytes();
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
            keychain: bdk::KeychainKind::External,
        }
    }
}

/// Result of running the coin selection algorithm succesfully.
#[derive(Debug)]
pub struct Output {
    pub coins: Vec<Utxo>,
    pub target_amount: Amount,
    pub recommended_fee: Amount,
}

impl Output {
    pub fn recommended_change(&self) -> Amount {
        self.selected_amount() - self.target_amount - self.recommended_fee
    }

    pub fn selected_amount(&self) -> Amount {
        let amount = self.coins.iter().fold(0, |acc, utxo| acc + utxo.value);
        Amount::from_sat(amount)
    }
}

/// Return the maximum weight of a satisfying witness.
///
/// Only supports P2PK, P2PKH and P2WPKH.
fn max_satisfaction_weight(script_pubkey: &Script) -> Option<usize> {
    if script_pubkey.is_p2pk() {
        Some(4 * (1 + 73))
    } else if script_pubkey.is_p2pkh() {
        Some(4 * (1 + 73 + 34))
    } else if script_pubkey.is_v0_p2wpkh() {
        Some(4 + 1 + 73 + 34)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elements::{Address, Txid};
    use std::str::FromStr;

    #[test]
    fn trivial_coin_selection() {
        let utxo = Utxo {
            outpoint: OutPoint {
                txid: Txid::default(),
                vout: 0,
            },
            value: 100_000_000,
            script_pubkey: Address::from_str("ert1qxzlkf3t275hwszualaf35spcfuq4s5tqtxj4tl")
                .unwrap()
                .script_pubkey(),
            asset: AssetId::default(),
        };

        let target_amount = Amount::from_sat(90_000_000);
        let selection = coin_select(vec![utxo.clone()], target_amount, 1.0, Amount::ZERO).unwrap();

        assert!(selection.coins.len() == 1);
        assert!(selection.coins.contains(&utxo));

        assert_eq!(
            selection.selected_amount() - target_amount - selection.recommended_fee,
            selection.recommended_change()
        );
    }
}

/// A placeholder for the `database` argument required by
/// `CoinSelectionAlgorithm::coin_select`, but which is never actually
/// used in the trait implementation.
struct DummyDb;

impl Database for DummyDb {
    fn check_descriptor_checksum<B: AsRef<[u8]>>(
        &mut self,
        _script_type: bdk::KeychainKind,
        _bytes: B,
    ) -> Result<(), bdk::Error> {
        todo!()
    }

    fn iter_script_pubkeys(
        &self,
        _script_type: Option<bdk::KeychainKind>,
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
        _script_type: bdk::KeychainKind,
        _child: u32,
    ) -> Result<Option<bdk::bitcoin::Script>, bdk::Error> {
        todo!()
    }

    fn get_path_from_script_pubkey(
        &self,
        _script: &bdk::bitcoin::Script,
    ) -> Result<Option<(bdk::KeychainKind, u32)>, bdk::Error> {
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

    fn get_last_index(&self, _script_type: bdk::KeychainKind) -> Result<Option<u32>, bdk::Error> {
        todo!()
    }

    fn increment_last_index(&mut self, _script_type: bdk::KeychainKind) -> Result<u32, bdk::Error> {
        todo!()
    }
}

impl BatchOperations for DummyDb {
    fn set_script_pubkey(
        &mut self,
        _script: &bdk::bitcoin::Script,
        _script_type: bdk::KeychainKind,
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
        _script_type: bdk::KeychainKind,
        _value: u32,
    ) -> Result<(), bdk::Error> {
        todo!()
    }

    fn del_script_pubkey_from_path(
        &mut self,
        _script_type: bdk::KeychainKind,
        _child: u32,
    ) -> Result<Option<bdk::bitcoin::Script>, bdk::Error> {
        todo!()
    }

    fn del_path_from_script_pubkey(
        &mut self,
        _script: &bdk::bitcoin::Script,
    ) -> Result<Option<(bdk::KeychainKind, u32)>, bdk::Error> {
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

    fn del_last_index(
        &mut self,
        _script_type: bdk::KeychainKind,
    ) -> Result<Option<u32>, bdk::Error> {
        todo!()
    }
}
