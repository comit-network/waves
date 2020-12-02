// Rust Elements Library
// Written in 2018 by
//   Andrew Poelstra <apoelstra@blockstream.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Rust Elements Library
//!
//! Extensions to `rust-bitcoin` to support deserialization and serialization
//! of Elements transactions and blocks.
//!

// Coding conventions
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
// #![deny(missing_docs)]

#[macro_use]
pub extern crate bitcoin_hashes;

#[cfg(feature = "serde")]
extern crate serde_crate as serde;

#[macro_use]
mod internal_macros;
pub mod address;
pub mod blech32;
mod block;
pub mod confidential;
pub mod dynafed;
pub mod encode;
mod fast_merkle_root;
pub mod hash_types;
pub mod issuance;
pub mod opcodes;
pub mod script;
pub mod slip77;
mod transaction;
pub mod wally;

// re-export bitcoin deps which we re-use
pub use bitcoin::{bech32, hashes, secp256k1};
// export everything at the top level so it can be used as `elements::Transaction` etc.
pub use ::bitcoin::consensus::encode::VarInt;
pub use address::{Address, AddressError, AddressParams};
pub use bitcoin;
pub use block::{Block, BlockHeader, ExtData as BlockExtData};
pub use fast_merkle_root::fast_merkle_root;
pub use hash_types::*;
pub use issuance::{AssetId, ContractHash};
pub use script::Script;
pub use transaction::{
    AssetIssuance, ConfidentialAssetIssuance, ConfidentialTxOut, ExplicitAsset,
    ExplicitAssetIssuance, ExplicitTxOut, ExplicitValue, OutPoint, PeginData, PegoutData,
    Transaction, TxIn, TxInWitness, TxOut, TxOutWitness, UnblindedTxOut,
};
