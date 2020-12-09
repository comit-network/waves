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

use crate::encode::{self, Decodable, Encodable};
use bitcoin::secp256k1::{
    rand::{CryptoRng, Rng, RngCore},
    PublicKey, Secp256k1, SecretKey, Signing,
};
use hex::{FromHex, FromHexError};
use std::{fmt, io};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Helper macro to implement various things for the various confidential
// commitment types
macro_rules! impl_confidential_commitment {
    ($name:ident, $prefixA:expr, $prefixB:expr) => {
        impl $name {
            pub const fn is_valid_prefix(tag: u8) -> bool {
                tag == $prefixA || tag == $prefixB
            }

            pub fn from_commitment(tag: u8, xcoor: &[u8]) -> Result<Self, encode::Error> {
                if xcoor.len() != 32 {
                    return Err(encode::Error::ParseFailed(
                        "x-coordinate of commitment must be 32 bytes long",
                    ));
                }

                if !Self::is_valid_prefix(tag) {
                    return Err(encode::Error::InvalidTag {
                        expected: $prefixA,
                        got: tag,
                    });
                }
                let mut commitment = [0u8; 33];
                commitment[0] = tag;
                commitment[1..].copy_from_slice(&xcoor);

                Ok(Self(commitment))
            }

            pub fn from_slice(commitment: &[u8]) -> Result<$name, encode::Error> {
                Self::from_commitment(commitment[0], &commitment[1..])
            }

            pub fn commitment(&self) -> [u8; 33] {
                self.0
            }

            pub fn encoded_length(&self) -> usize {
                33
            }
        }

        impl hex::FromHex for $name {
            type Error = encode::Error;

            fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
                let bytes = Vec::<u8>::from_hex(hex)
                    .map_err(|_| encode::Error::ParseFailed("failed to parse as hex"))?; // TODO: Proper error handling

                Ok($name::from_slice(&bytes)?)
            }
        }

        impl Encodable for $name {
            fn consensus_encode<S: io::Write>(&self, mut s: S) -> Result<usize, encode::Error> {
                self.0.consensus_encode(&mut s)
            }
        }

        impl Decodable for $name {
            fn consensus_decode<D: io::BufRead>(mut d: D) -> Result<$name, encode::Error> {
                let bytes = <[u8; 33]>::consensus_decode(&mut d)?;

                Ok(Self::from_commitment(bytes[0], &bytes[1..])?)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                for b in self.0.iter() {
                    write!(f, "{:02x}", b)?;
                }
                Ok(())
            }
        }

        #[cfg(feature = "serde")]
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                use serde::ser::SerializeSeq;

                let mut seq = s.serialize_seq(Some(33))?;
                seq.serialize_element(self.0.as_ref())?;
                seq.end()
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                use serde::de::{Error, SeqAccess, Visitor};
                struct CommitVisitor;

                impl<'de> Visitor<'de> for CommitVisitor {
                    type Value = $name;

                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("a committed value")
                    }

                    fn visit_seq<A: SeqAccess<'de>>(
                        self,
                        mut access: A,
                    ) -> Result<Self::Value, A::Error> {
                        let prefix: u8 = if let Some(x) = access.next_element()? {
                            x
                        } else {
                            return Err(A::Error::custom("missing prefix"));
                        };

                        if prefix != $prefixA && prefix != $prefixB {
                            return Err(A::Error::custom(format!("invalid prefix {}", prefix)));
                        }

                        let xcoor = access
                            .next_element::<[u8; 32]>()?
                            .ok_or_else(|| A::Error::custom("missing x-coordinate"))?;

                        Ok($name::from_commitment(prefix, &xcoor).map_err(A::Error::custom)?)
                    }
                }

                d.deserialize_seq(CommitVisitor)
            }
        }
    };
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct AssetCommitment([u8; 33]);

#[cfg(feature = "wally-sys")]
impl AssetCommitment {
    pub fn new(asset: crate::AssetId, bf: AssetBlindingFactor) -> Self {
        crate::wally::asset_generator_from_bytes(&asset, &bf)
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ValueCommitment([u8; 33]);

#[cfg(feature = "wally-sys")]
impl ValueCommitment {
    pub fn new(value: u64, asset: AssetCommitment, bf: ValueBlindingFactor) -> Self {
        crate::wally::asset_value_commitment(value, bf, asset)
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Nonce([u8; 33]);

impl Nonce {
    pub fn new<R: RngCore + CryptoRng, C: Signing>(
        rng: &mut R,
        secp: &Secp256k1<C>,
    ) -> (Self, SecretKey) {
        let secret_key = SecretKey::new(rng);
        let commitment = PublicKey::from_secret_key(&secp, &secret_key);

        (Self(commitment.serialize()), secret_key)
    }
}

impl_confidential_commitment!(AssetCommitment, 0x0a, 0x0b);
impl_confidential_commitment!(ValueCommitment, 0x08, 0x09);
impl_confidential_commitment!(Nonce, 0x02, 0x03);

impl From<PublicKey> for Nonce {
    fn from(public_key: PublicKey) -> Self {
        Nonce(public_key.serialize())
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ValueBlindingFactor([u8; 32]);

#[cfg(feature = "wally-sys")]
impl ValueBlindingFactor {
    pub fn random<R: Rng>(rng: &mut R) -> Self {
        Self(*SecretKey::new(rng).as_ref())
    }

    pub fn last(
        value: u64,
        abf: AssetBlindingFactor,
        inputs: &[(u64, AssetBlindingFactor, ValueBlindingFactor)],
        outputs: &[(u64, AssetBlindingFactor, ValueBlindingFactor)],
    ) -> Self {
        let amounts = {
            let input_amounts = inputs.iter().map(|(amount, _, _)| amount);
            let output_amounts = outputs
                .iter()
                .map(|(amount, _, _)| amount)
                .chain(std::iter::once(&value));

            input_amounts.chain(output_amounts)
        };
        let abfs = {
            let input_abfs = inputs.iter().map(|(_, abf, _)| abf);
            let output_abfs = outputs
                .iter()
                .map(|(_, abf, _)| abf)
                .chain(std::iter::once(&abf));

            input_abfs.chain(output_abfs)
        };
        let vbfs = {
            let input_vbfs = inputs.iter().map(|(_, _, vbf)| vbf);
            let output_vbfs = outputs.iter().map(|(_, _, vbf)| vbf);

            input_vbfs.chain(output_vbfs)
        };

        crate::wally::asset_final_vbf(amounts, inputs.len(), abfs, vbfs)
    }

    pub fn into_inner(self) -> [u8; 32] {
        self.0
    }
}

impl FromHex for ValueBlindingFactor {
    type Error = FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Ok(Self(FromHex::from_hex(hex)?))
    }
}

impl From<[u8; 32]> for ValueBlindingFactor {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct AssetBlindingFactor([u8; 32]);

impl AssetBlindingFactor {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        Self(*SecretKey::new(rng).as_ref())
    }

    pub fn into_inner(self) -> [u8; 32] {
        self.0
    }
}

impl FromHex for AssetBlindingFactor {
    type Error = FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Ok(Self(FromHex::from_hex(hex)?))
    }
}

impl From<[u8; 32]> for AssetBlindingFactor {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commitments() {
        let x = ValueCommitment::from_commitment(0x08, &[1; 32]).unwrap();
        let mut commitment = x.commitment();
        assert_eq!(x, ValueCommitment::from_slice(&commitment[..]).unwrap());
        commitment[0] = 42;
        assert!(ValueCommitment::from_slice(&commitment[..]).is_err());

        let x = AssetCommitment::from_commitment(0x0a, &[1; 32]).unwrap();
        let mut commitment = x.commitment();
        assert_eq!(x, AssetCommitment::from_slice(&commitment[..]).unwrap());
        commitment[0] = 42;
        assert!(AssetCommitment::from_slice(&commitment[..]).is_err());

        let x = Nonce::from_commitment(0x02, &[1; 32]).unwrap();
        let mut commitment = x.commitment();
        assert_eq!(x, Nonce::from_slice(&commitment[..]).unwrap());
        commitment[0] = 42;
        assert!(Nonce::from_slice(&commitment[..]).is_err());
    }
}
