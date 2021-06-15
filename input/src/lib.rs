use anyhow::Result;
use elements::{
    secp256k1_zkp::{Secp256k1, SecretKey, Verification},
    OutPoint, TxOut, TxOutSecrets,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Input {
    pub txin: OutPoint,
    pub original_txout: TxOut,
    pub blinding_key: SecretKey,
}

impl Input {
    pub fn into_unblinded_input<C>(self, secp: &Secp256k1<C>) -> Result<UnblindedInput>
    where
        C: Verification,
    {
        let txin = self.txin;
        let txout = self.original_txout;
        let secrets = txout.unblind(secp, self.blinding_key)?;

        Ok(UnblindedInput {
            txin,
            txout,
            secrets,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UnblindedInput {
    pub txin: OutPoint,
    pub txout: TxOut,
    pub secrets: TxOutSecrets,
}
