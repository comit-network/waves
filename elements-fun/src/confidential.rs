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

//! # Confidential Commitments
//!
//! Structures representing Pedersen commitments of various types
//!

use crate::{
    encode::{Decodable, Encodable},
    AssetId,
};
use bitcoin::hashes::{sha256d, Hash};
use secp256k1_zkp::{
    compute_adaptive_blinding_factor,
    ecdh::SharedSecret,
    rand::{CryptoRng, Rng, RngCore},
    CommitmentSecrets, Error, Generator, PedersenCommitment, PublicKey, Secp256k1, SecretKey,
    Signing,
};
use std::io;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(crate = "serde_crate")
)]
pub struct AssetGenerator(pub(crate) Generator);

impl AssetGenerator {
    pub fn new<C: Signing>(secp: &Secp256k1<C>, asset: AssetId, bf: AssetBlindingFactor) -> Self {
        Self(Generator::new_blinded(
            secp,
            asset.into_tag(),
            bf.into_inner(),
        ))
    }

    pub fn encoded_length(&self) -> usize {
        33
    }
}

impl Encodable for AssetGenerator {
    fn consensus_encode<W: io::Write>(&self, mut e: W) -> Result<usize, crate::encode::Error> {
        e.write_all(&self.0.serialize())?;

        Ok(33)
    }
}

impl Decodable for AssetGenerator {
    fn consensus_decode<D: io::BufRead>(d: D) -> Result<Self, crate::encode::Error> {
        let bytes = <[u8; 33]>::consensus_decode(d)?;

        Ok(Self(Generator::from_slice(&bytes)?))
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(crate = "serde_crate")
)]
pub struct ValueCommitment(pub(crate) PedersenCommitment);

impl ValueCommitment {
    pub fn new<C: Signing>(
        secp: &Secp256k1<C>,
        value: u64,
        asset: AssetGenerator,
        bf: ValueBlindingFactor,
    ) -> Self {
        Self(PedersenCommitment::new(secp, value, bf.0, asset.0))
    }

    pub fn encoded_length(&self) -> usize {
        33
    }
}

impl Encodable for ValueCommitment {
    fn consensus_encode<W: io::Write>(&self, mut e: W) -> Result<usize, crate::encode::Error> {
        e.write_all(&self.0.serialize())?;

        Ok(33)
    }
}

impl Decodable for ValueCommitment {
    fn consensus_decode<D: io::BufRead>(d: D) -> Result<Self, crate::encode::Error> {
        let bytes = <[u8; 33]>::consensus_decode(d)?;

        Ok(Self(PedersenCommitment::from_slice(&bytes)?))
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(crate = "serde_crate")
)]
pub struct Nonce(pub(crate) PublicKey);

impl Nonce {
    pub fn new<R: RngCore + CryptoRng, C: Signing>(
        rng: &mut R,
        secp: &Secp256k1<C>,
        receiver_blinding_pk: &PublicKey,
    ) -> (Self, SecretKey) {
        let sender_sk = SecretKey::new(rng);
        let sender_pk = PublicKey::from_secret_key(&secp, &sender_sk);

        let shared_secret = Self::make_shared_secret(receiver_blinding_pk, &sender_sk);

        (Self(sender_pk), shared_secret)
    }

    pub fn shared_secret(&self, receiver_blinding_sk: &SecretKey) -> SecretKey {
        let sender_pk = self.0;
        Self::make_shared_secret(&sender_pk, receiver_blinding_sk)
    }

    /// Create the shared secret.
    fn make_shared_secret(pk: &PublicKey, sk: &SecretKey) -> SecretKey {
        let shared_secret = SharedSecret::new_with_hash(pk, sk, |x, y| {
            // Yes, what follows is the compressed representation of a Bitcoin public key.
            // However, this is more by accident then by design, see here: https://github.com/rust-bitcoin/rust-secp256k1/pull/255#issuecomment-744146282

            let mut dh_secret = [0u8; 33];
            dh_secret[0] = if y.last().unwrap() % 2 == 0 {
                0x02
            } else {
                0x03
            };
            dh_secret[1..].copy_from_slice(&x);

            sha256d::Hash::hash(&dh_secret).into_inner().into()
        });

        SecretKey::from_slice(&shared_secret.as_ref()[..32]).expect("always has exactly 32 bytes")
    }

    pub fn encoded_length(&self) -> usize {
        33
    }
}

impl From<PublicKey> for Nonce {
    fn from(public_key: PublicKey) -> Self {
        Nonce(public_key)
    }
}

impl Encodable for Nonce {
    fn consensus_encode<W: io::Write>(&self, mut e: W) -> Result<usize, crate::encode::Error> {
        e.write_all(&self.0.serialize())?;

        Ok(33)
    }
}

impl Decodable for Nonce {
    fn consensus_decode<D: io::BufRead>(d: D) -> Result<Self, crate::encode::Error> {
        let bytes = <[u8; 33]>::consensus_decode(d)?;

        Ok(Self(PublicKey::from_slice(&bytes)?))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ValueBlindingFactor(pub(crate) SecretKey);

impl ValueBlindingFactor {
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        Self(SecretKey::new(rng))
    }

    pub fn last<C: Signing>(
        secp: &Secp256k1<C>,
        value: u64,
        abf: AssetBlindingFactor,
        inputs: &[(u64, AssetBlindingFactor, ValueBlindingFactor)],
        outputs: &[(u64, AssetBlindingFactor, ValueBlindingFactor)],
    ) -> Self {
        let set_a = inputs
            .iter()
            .copied()
            .map(|(value, abf, vbf)| CommitmentSecrets {
                value,
                value_blinding_factor: vbf.0,
                generator_blinding_factor: abf.into_inner(),
            })
            .collect::<Vec<_>>();
        let set_b = outputs
            .iter()
            .copied()
            .map(|(value, abf, vbf)| CommitmentSecrets {
                value,
                value_blinding_factor: vbf.0,
                generator_blinding_factor: abf.into_inner(),
            })
            .collect::<Vec<_>>();

        Self(compute_adaptive_blinding_factor(
            secp, value, abf.0, &set_a, &set_b,
        ))
    }
}

// impl FromHex for ValueBlindingFactor {
//     type Error = FromHexError;

//     fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
//         Ok(Self(FromHex::from_hex(hex)?))
//     }
// }

// impl From<[u8; 32]> for ValueBlindingFactor {
//     fn from(bytes: [u8; 32]) -> Self {
//         Self(bytes)
//     }
// }

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct AssetBlindingFactor(pub(crate) SecretKey);

impl AssetBlindingFactor {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        Self(SecretKey::new(rng))
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Self(SecretKey::from_slice(bytes)?))
    }

    pub fn into_inner(self) -> SecretKey {
        self.0
    }
}
