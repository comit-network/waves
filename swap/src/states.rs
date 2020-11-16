use bitcoin::Amount;
use elements_fun::confidential::Asset;
use elements_fun::Address;
use elements_fun::AssetId;
use elements_fun::OutPoint;
use elements_fun::TxIn;
use elements_fun::TxOut;
use rand::CryptoRng;
use rand::RngCore;
use secp256k1::SecretKey;
use secp256k1::Signature;

use crate::unblind_asset_from_txout;

/// Sent from Alice to Bob, assuming Alice has bitcoin.
pub struct Message0 {
    pub input: TxIn,
    pub asset_id_in: AssetId,
    pub asset_id_commitment_in: Asset,
    pub abf_in: SecretKey,
    pub address_redeem: Address,
    pub abf_redeem: SecretKey,
    pub vbf_redeem: SecretKey,
    pub address_change: Address,
    pub abf_change: SecretKey,
    pub vbf_change: SecretKey,
    pub fee: Amount,
}

/// Sent from Bob to Alice.
pub struct Message1 {
    pub input: TxIn,
    pub asset_id_in: AssetId,
    pub asset_id_commitment_in: Asset,
    pub abf_in: SecretKey,
    pub address_redeem: Address,
    pub abf_redeem: SecretKey,
    pub vbf_redeem: SecretKey,
    pub signature: Signature,
}

pub struct Alice0 {
    pub amount_want: Amount,
    pub input: TxIn,
    pub asset_id_have: AssetId,
    pub asset_id_want: AssetId,
    pub asset_id_commitment_in: Asset,
    pub abf_in: SecretKey,
    pub address_redeem: Address,
    pub abf_redeem: SecretKey,
    pub vbf_redeem: SecretKey,
    pub address_change: Address,
    pub abf_change: SecretKey,
    pub vbf_change: SecretKey,
    pub fee: Amount,
}

impl Alice0 {
    pub fn new<R>(
        rng: &mut R,
        amount_want: Amount,
        // TODO: Define struct
        input: (OutPoint, TxOut),
        input_blinding_sk: SecretKey,
        asset_id_want: AssetId,
        address_redeem: Address,
        address_change: Address,
        fee: Amount,
    ) -> Self
    where
        R: RngCore + CryptoRng,
    {
        let (asset_id_have, asset_id_commitment_in, abf_in, _vbf_in, _amount_in) =
            unblind_asset_from_txout(input.1, input_blinding_sk);

        let input = TxIn {
            previous_output: input.0,
            is_pegin: false,
            has_issuance: false,
            script_sig: Default::default(),
            sequence: 0xFFFF_FFFF,
            asset_issuance: Default::default(),
            witness: Default::default(),
        };

        let abf_redeem = SecretKey::new(rng);
        let vbf_redeem = SecretKey::new(rng);

        let abf_change = SecretKey::new(rng);
        let vbf_change = SecretKey::new(rng);

        Self {
            amount_want,
            input,
            asset_id_have,
            asset_id_want,
            asset_id_commitment_in,
            abf_in,
            address_redeem,
            abf_redeem,
            vbf_redeem,
            address_change,
            abf_change,
            vbf_change,
            fee,
        }
    }
}
