use anyhow::Result;
use elements::{
    secp256k1_zkp::{Secp256k1, SecretKey, Verification},
    TxIn, TxOut, TxOutSecrets,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Input {
    pub txin: TxIn,
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
    pub txin: TxIn,
    pub txout: TxOut,
    pub secrets: TxOutSecrets,
}
