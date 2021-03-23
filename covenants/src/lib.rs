use std::future::Future;

use anyhow::{bail, Context, Result};
use elements::{bitcoin::util::psbt::serialize::Serialize, confidential::Nonce, hashes::Hash};
use elements::{
    bitcoin::{Amount, Network, PrivateKey, PublicKey},
    TxOutWitness,
};
use elements::{
    confidential::{Asset, Value},
    AddressParams,
};
use elements::{encode::Encodable, hashes::sha256d, SigHashType};
use elements::{opcodes::all::*, secp256k1::Signature};
use elements::{script::Builder, OutPoint};
use elements::{secp256k1::rand::thread_rng, sighash::SigHashCache};
use elements::{
    secp256k1::{SecretKey, SECP256K1},
    TxInWitness,
};
use elements::{Address, AssetId, Script, Transaction, TxIn, TxOut};

#[cfg(test)]
mod happy_test;

pub struct LoanRequest {
    collateral_amount: Amount,
    collateral_tx_ins: Vec<TxIn>,
    collateral_change_tx_out: TxOut,
    tx_fee: Amount,
    borrower_pk: PublicKey,
    timelock: u64,
    borrower_address: Address,
}

pub struct LoanResponse {
    transaction: Transaction,
    lender_pk: PublicKey,
    lender_address: Address,
    timelock: u64,
}

pub struct Borrower0 {
    keypair: (SecretKey, PublicKey),
    address: Address,
    collateral_amount: Amount,
    collateral_tx_ins: Vec<TxIn>,
    collateral_change_tx_out: TxOut,
    tx_fee: Amount,
    timelock: u64,
    bitcoin_asset_id: AssetId,
    usdt_asset_id: AssetId,
}

impl Borrower0 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address: Address,
        collateral_amount: Amount,
        collateral_inputs: Vec<Input>,
        tx_fee: Amount,
        timelock: u64,
        bitcoin_asset_id: AssetId,
        usdt_asset_id: AssetId,
    ) -> Result<Self> {
        let keypair = make_keypair();

        let collateral_input_amount = &collateral_inputs
            .iter()
            .fold(Amount::ZERO, |sum, input| sum + input.amount);
        let change_amount = collateral_input_amount
            .checked_sub(collateral_amount)
            .map(|a| a.checked_sub(tx_fee))
            .flatten()
            .with_context(|| {
                format!(
                    "cannot pay for output {} and fee {} with input {}",
                    collateral_amount, tx_fee, collateral_input_amount,
                )
            })?;

        let collateral_change_tx_out = TxOut {
            asset: Asset::Explicit(bitcoin_asset_id),
            value: Value::Explicit(change_amount.as_sat()),
            nonce: Nonce::Null,
            script_pubkey: address.script_pubkey(),
            witness: TxOutWitness::default(),
        };

        let collateral_tx_ins = collateral_inputs
            .iter()
            .map(|input| input.tx_in.clone())
            .collect();

        Ok(Self {
            keypair,
            address,
            collateral_amount,
            collateral_tx_ins,
            collateral_change_tx_out,
            timelock,
            tx_fee,
            bitcoin_asset_id,
            usdt_asset_id,
        })
    }

    pub fn loan_request(&self) -> LoanRequest {
        LoanRequest {
            collateral_amount: self.collateral_amount,
            collateral_tx_ins: self.collateral_tx_ins.clone(),
            collateral_change_tx_out: self.collateral_change_tx_out.clone(),
            tx_fee: self.tx_fee,
            borrower_pk: self.keypair.1,
            timelock: self.timelock,
            borrower_address: self.address.clone(),
        }
    }

    pub fn interpret(self, loan_response: LoanResponse) -> Result<Borrower1> {
        let transaction = loan_response.transaction;

        let principal_tx_out = transaction
            .output
            .iter()
            .find(|out| {
                out.asset.explicit().unwrap() == self.usdt_asset_id
                    && out.script_pubkey == self.address.script_pubkey()
            })
            .context("no principal txout")?;

        let (collateral_script, repayment_tx_out) = loan_contract(
            self.keypair.1,
            loan_response.lender_pk,
            Amount::from_sat(principal_tx_out.value.explicit().unwrap()),
            &loan_response.lender_address,
            loan_response.timelock,
            self.usdt_asset_id,
        );
        let collateral_address = Address::p2wsh(&collateral_script, None, &AddressParams::ELEMENTS);
        let collateral_script_pubkey = collateral_address.script_pubkey();

        transaction
            .output
            .iter()
            .find(|out| {
                out.asset.explicit().unwrap() == self.bitcoin_asset_id
                    && out.value.explicit().unwrap() == self.collateral_amount.as_sat()
                    && out.script_pubkey == collateral_script_pubkey
            })
            .context("no collateral txout")?;

        transaction
            .output
            .iter()
            .find(|out| out == &&self.collateral_change_tx_out)
            .context("no collateral change txout")?;

        Ok(Borrower1 {
            keypair: self.keypair,
            loan_transaction: transaction.clone(),
            collateral_amount: self.collateral_amount,
            collateral_script,
            principal_tx_out: principal_tx_out.clone(),
            address: self.address,
            repayment_tx_out,
            bitcoin_asset_id: self.bitcoin_asset_id,
            usdt_asset_id: self.usdt_asset_id,
        })
    }
}

pub struct Borrower1 {
    keypair: (SecretKey, PublicKey),
    loan_transaction: Transaction,
    collateral_amount: Amount,
    collateral_script: Script,
    principal_tx_out: TxOut,
    address: Address,
    repayment_tx_out: TxOut,
    bitcoin_asset_id: AssetId,
    usdt_asset_id: AssetId,
}

impl Borrower1 {
    pub async fn sign<S, F>(&self, signer: S) -> Result<Transaction>
    where
        S: FnOnce(Transaction) -> F,
        F: Future<Output = Result<Transaction>>,
    {
        signer(self.loan_transaction.clone()).await
    }

    pub async fn loan_repayment_transaction<C, CF, S, SF>(
        &self,
        coin_selector: C,
        signer: S,
        tx_fee: Amount,
    ) -> Result<Transaction>
    where
        C: FnOnce(Amount, AssetId) -> CF,
        CF: Future<Output = Result<Vec<Input>>>,
        S: FnOnce(Transaction) -> SF,
        SF: Future<Output = Result<Transaction>>,
    {
        let loan_transaction = self.loan_transaction.clone();
        let loan_txid = loan_transaction.txid();

        // construct collateral input
        let collateral_address =
            Address::p2wsh(&self.collateral_script, None, &AddressParams::ELEMENTS);
        let collateral_script_pubkey = collateral_address.script_pubkey();
        let vout = self
            .loan_transaction
            .output
            .iter()
            .position(|out| out.script_pubkey == collateral_script_pubkey)
            .context("no collateral txout")?;

        let collateral_input = TxIn {
            previous_output: OutPoint {
                txid: loan_txid,
                vout: vout as u32,
            },
            is_pegin: false,
            has_issuance: false,
            script_sig: Default::default(),
            sequence: 0,
            asset_issuance: Default::default(),
            witness: Default::default(),
        };

        // construct repayment input and repayment change output
        let (mut repayment_inputs, repayment_change) = {
            let repayment_amount =
                Amount::from_sat(self.principal_tx_out.value.explicit().unwrap());
            let inputs = coin_selector(repayment_amount, self.usdt_asset_id).await?;

            let input_amount = inputs
                .iter()
                .fold(Amount::ZERO, |acc, input| acc + input.amount);
            let inputs = inputs.into_iter().map(|input| input.tx_in).collect();

            let change_amount = input_amount
                .checked_sub(repayment_amount)
                .with_context(|| {
                    format!(
                        "cannot pay for output {} with input {}",
                        repayment_amount, input_amount,
                    )
                })?;

            let change_output = match change_amount {
                Amount::ZERO => None,
                _ => Some(TxOut {
                    asset: Asset::Explicit(self.usdt_asset_id),
                    value: Value::Explicit(change_amount.as_sat()),
                    nonce: Nonce::Null,
                    script_pubkey: self.address.script_pubkey(),
                    witness: TxOutWitness::default(),
                }),
            };

            (inputs, change_output)
        };

        let collateral_output = TxOut {
            asset: Asset::Explicit(self.bitcoin_asset_id),
            value: Value::Explicit((self.collateral_amount - tx_fee).as_sat()),
            nonce: Default::default(),
            script_pubkey: self.address.script_pubkey(),
            witness: Default::default(),
        };

        let tx_fee_output = TxOut {
            asset: Asset::Explicit(self.bitcoin_asset_id),
            value: Value::Explicit(tx_fee.as_sat()),
            nonce: Default::default(),
            script_pubkey: Default::default(),
            witness: Default::default(),
        };

        let mut tx_ins = vec![collateral_input];
        tx_ins.append(&mut repayment_inputs);

        let mut tx_outs = vec![
            self.repayment_tx_out.clone(),
            collateral_output,
            tx_fee_output,
        ];
        if let Some(repayment_change) = repayment_change {
            tx_outs.push(repayment_change)
        }

        let mut tx = Transaction {
            version: 2,
            lock_time: 0,
            input: tx_ins,
            output: tx_outs,
        };

        // fulfill collateral input covenant script
        {
            let sighash = SigHashCache::new(&tx).segwitv0_sighash(
                0,
                &self.collateral_script.clone(),
                Value::Explicit(self.collateral_amount.as_sat()),
                SigHashType::All,
            );

            let sig = SECP256K1.sign(
                &elements::secp256k1::Message::from(sighash),
                &self.keypair.0,
            );

            tx.input[0].witness = TxInWitness {
                amount_rangeproof: vec![],
                inflation_keys_rangeproof: vec![],
                script_witness: RepaymentWitnessStack::new(
                    sig,
                    self.keypair.1,
                    self.collateral_amount.as_sat(),
                    &tx,
                    self.collateral_script.clone(),
                )
                .unwrap()
                .serialise()
                .unwrap(),
                pegin_witness: vec![],
            };
        };

        let tx = signer(tx).await?;

        Ok(tx)
    }
}

pub struct Lender0 {
    pub keypair: (SecretKey, PublicKey),
    pub principal_inputs: Vec<Input>,
    pub address: Address,
    pub bitcoin_asset_id: AssetId,
    pub usdt_asset_id: AssetId,
}

impl Lender0 {
    pub fn new(
        bitcoin_asset_id: AssetId,
        usdt_asset_id: AssetId,
        // TODO: Here we assume that the wallet is giving us _all_ the
        // inputs available. It would be better to coin-select these
        // as soon as we know the principal amount after receiving the
        // loan request
        principal_inputs: Vec<Input>,
        address: Address,
    ) -> Self {
        let keypair = make_keypair();

        Self {
            bitcoin_asset_id,
            keypair,
            address,
            usdt_asset_id,
            principal_inputs,
        }
    }

    pub fn interpret(self, loan_request: LoanRequest) -> Lender1 {
        let principal_amount = Lender0::calc_principal_amount(&loan_request);

        let (_, lender_pk) = self.keypair;
        let (collateral_script, _) = loan_contract(
            loan_request.borrower_pk,
            lender_pk,
            principal_amount,
            &self.address,
            loan_request.timelock,
            self.usdt_asset_id,
        );
        let collateral_address = Address::p2wsh(&collateral_script, None, &AddressParams::ELEMENTS);
        let collateral_tx_out = TxOut {
            asset: Asset::Explicit(self.bitcoin_asset_id),
            value: Value::Explicit(loan_request.collateral_amount.as_sat()),
            nonce: Default::default(),
            script_pubkey: collateral_address.script_pubkey(),
            witness: Default::default(),
        };

        let principal_tx_out = TxOut {
            asset: Asset::Explicit(self.usdt_asset_id),
            value: Value::Explicit(principal_amount.as_sat()),
            nonce: Default::default(),
            script_pubkey: loan_request.borrower_address.script_pubkey(),
            witness: Default::default(),
        };

        let principal_input_amount = self
            .principal_inputs
            .iter()
            .fold(Amount::ZERO, |sum, input| sum + input.amount);
        let principal_change_tx_out = TxOut {
            asset: Asset::Explicit(self.usdt_asset_id),
            value: Value::Explicit(principal_input_amount.as_sat() - principal_amount.as_sat()),
            nonce: Default::default(),
            script_pubkey: self.address.script_pubkey(),
            witness: Default::default(),
        };

        let collateral_change_tx_out = loan_request.collateral_change_tx_out;
        let tx_fee_tx_out = TxOut::new_fee(loan_request.tx_fee.as_sat(), self.bitcoin_asset_id);

        let mut tx_ins = self
            .principal_inputs
            .iter()
            .map(|input| input.tx_in.clone())
            .collect::<Vec<_>>();
        let mut collateral_tx_ins = loan_request.collateral_tx_ins;
        tx_ins.append(&mut collateral_tx_ins);
        let loan_transaction = Transaction {
            version: 2,
            lock_time: 0,
            input: tx_ins,
            output: vec![
                collateral_tx_out,
                principal_tx_out,
                principal_change_tx_out,
                collateral_change_tx_out,
                tx_fee_tx_out,
            ],
        };

        Lender1 {
            keypair: self.keypair,
            address: self.address,
            timelock: loan_request.timelock,
            loan_transaction,
        }
    }

    fn calc_principal_amount(loan_request: &LoanRequest) -> Amount {
        Amount::from_sat(loan_request.collateral_amount.as_sat() / 2)
    }
}

pub struct Lender1 {
    pub keypair: (SecretKey, PublicKey),
    pub address: Address,
    pub timelock: u64,
    pub loan_transaction: Transaction,
}

impl Lender1 {
    pub fn loan_response(&self) -> LoanResponse {
        LoanResponse {
            transaction: self.loan_transaction.clone(),
            lender_pk: self.keypair.1,
            lender_address: self.address.clone(),
            timelock: self.timelock,
        }
    }

    pub async fn finalise_loan<S, F>(
        &self,
        loan_transaction: Transaction,
        signer: S,
    ) -> Result<Transaction>
    where
        S: FnOnce(Transaction) -> F,
        F: Future<Output = Result<Transaction>>,
    {
        if self.loan_transaction.txid() != loan_transaction.txid() {
            bail!("wrong loan transaction")
        }

        signer(loan_transaction).await
    }
}

fn loan_contract(
    borrower_pk: PublicKey,
    lender_pk: PublicKey,
    principal_amount: Amount,
    lender_address: &Address,
    timelock: u64,
    usdt_asset_id: AssetId,
) -> (Script, TxOut) {
    let repayment_output = TxOut {
        asset: Asset::Explicit(usdt_asset_id),
        value: Value::Explicit(principal_amount.as_sat()),
        nonce: Default::default(),
        script_pubkey: lender_address.script_pubkey(),
        witness: Default::default(),
    };

    let mut repayment_output_bytes = Vec::new();
    repayment_output
        .consensus_encode(&mut repayment_output_bytes)
        .unwrap();

    let script = Builder::new()
        .push_opcode(OP_IF)
        .push_opcode(OP_DEPTH)
        .push_opcode(OP_1SUB)
        .push_opcode(OP_PICK)
        .push_opcode(OP_PUSHNUM_1)
        .push_opcode(OP_CAT)
        .push_slice(&borrower_pk.serialize())
        .push_opcode(OP_CHECKSIGVERIFY)
        .push_slice(repayment_output_bytes.as_slice())
        .push_opcode(OP_2ROT)
        .push_int(5)
        .push_opcode(OP_ROLL)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_HASH256)
        .push_opcode(OP_ROT)
        .push_opcode(OP_ROT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_CAT)
        .push_opcode(OP_SHA256)
        .push_opcode(OP_SWAP)
        .push_opcode(OP_CHECKSIGFROMSTACK)
        .push_opcode(OP_ELSE)
        .push_int(timelock as i64)
        .push_opcode(OP_CLTV)
        .push_opcode(OP_DROP)
        .push_opcode(OP_DUP)
        .push_slice(&lender_pk.serialize())
        .push_opcode(OP_CHECKSIG)
        .push_opcode(OP_ENDIF)
        .into_script();

    (script, repayment_output)
}

struct RepaymentWitnessStack {
    sig: Signature,
    pk: PublicKey,
    tx_version: u32,
    hash_prev_out: sha256d::Hash,
    hash_sequence: sha256d::Hash,
    hash_issuances: sha256d::Hash,
    input: InputData,
    other_outputs: Vec<TxOut>,
    lock_time: u32,
    sighash_type: SigHashType,
}

struct InputData {
    previous_output: OutPoint,
    script: Script,
    value: Value,
    sequence: u32,
}

impl RepaymentWitnessStack {
    fn new(
        sig: Signature,
        pk: PublicKey,
        collateral_amount: u64,
        tx: &Transaction,
        script: Script,
    ) -> Result<Self> {
        let tx_version = tx.version;

        let hash_prev_out = {
            let mut enc = sha256d::Hash::engine();
            for txin in tx.input.iter() {
                txin.previous_output.consensus_encode(&mut enc)?;
            }

            sha256d::Hash::from_engine(enc)
        };

        let hash_sequence = {
            let mut enc = sha256d::Hash::engine();

            for txin in tx.input.iter() {
                txin.sequence.consensus_encode(&mut enc)?;
            }
            sha256d::Hash::from_engine(enc)
        };

        let hash_issuances = {
            let mut enc = sha256d::Hash::engine();
            for txin in tx.input.iter() {
                if txin.has_issuance() {
                    txin.asset_issuance.consensus_encode(&mut enc)?;
                } else {
                    0u8.consensus_encode(&mut enc)?;
                }
            }
            sha256d::Hash::from_engine(enc)
        };

        let input = {
            let input = &tx.input[0];
            let value = Value::Explicit(collateral_amount);
            InputData {
                previous_output: input.previous_output,
                script,
                value,
                sequence: input.sequence,
            }
        };

        let other_outputs = tx.output[1..].to_vec();

        let lock_time = tx.lock_time;

        let sighash_type = SigHashType::All;

        Ok(Self {
            sig,
            pk,
            tx_version,
            hash_prev_out,
            hash_sequence,
            hash_issuances,
            input,
            other_outputs,
            lock_time,
            sighash_type,
        })
    }

    fn serialise(&self) -> anyhow::Result<Vec<Vec<u8>>> {
        let if_flag = vec![0x01];

        let sig = self.sig.serialize_der().to_vec();

        let pk = self.pk.serialize().to_vec();

        let tx_version = {
            let mut writer = Vec::new();
            self.tx_version.consensus_encode(&mut writer)?;
            writer
        };

        // input specific values
        let (previous_out, script_0, script_1, script_2, value, sequence) = {
            let InputData {
                previous_output,
                script,
                value,
                sequence,
            } = &self.input;

            let third = script.len() / 3;

            (
                {
                    let mut writer = Vec::new();
                    previous_output.consensus_encode(&mut writer)?;
                    writer
                },
                {
                    let mut writer = Vec::new();
                    script.consensus_encode(&mut writer)?;
                    writer[..third].to_vec()
                },
                {
                    let mut writer = Vec::new();
                    script.consensus_encode(&mut writer)?;
                    writer[third..2 * third].to_vec()
                },
                {
                    let mut writer = Vec::new();
                    script.consensus_encode(&mut writer)?;
                    writer[2 * third..].to_vec()
                },
                {
                    let mut writer = Vec::new();
                    value.consensus_encode(&mut writer)?;
                    writer
                },
                {
                    let mut writer = Vec::new();
                    sequence.consensus_encode(&mut writer)?;
                    writer
                },
            )
        };

        // hashoutputs (only supporting SigHashType::All)
        let other_outputs = {
            let mut other_outputs = vec![];

            for txout in self.other_outputs.iter() {
                let mut output = Vec::new();
                txout.consensus_encode(&mut output)?;
                other_outputs.push(output)
            }

            if other_outputs.len() < 2 {
                bail!("insufficient outputs");
            }

            if other_outputs.len() == 2 {
                other_outputs.push(vec![])
            }

            other_outputs
        };

        let lock_time = {
            let mut writer = Vec::new();
            self.lock_time.consensus_encode(&mut writer)?;
            writer
        };

        let sighash_type = {
            let mut writer = Vec::new();
            self.sighash_type.as_u32().consensus_encode(&mut writer)?;
            writer
        };

        Ok(vec![
            sig,
            pk,
            tx_version,
            self.hash_prev_out.to_vec(),
            self.hash_sequence.to_vec(),
            self.hash_issuances.to_vec(),
            previous_out,
            script_0,
            script_1,
            script_2,
            value,
            sequence,
            other_outputs[0].clone(),
            other_outputs[1].clone(),
            other_outputs[2].clone(),
            lock_time,
            sighash_type,
            if_flag,
            self.input.script.clone().into_bytes(),
        ])
    }
}

#[derive(Debug)]
pub struct Input {
    pub amount: Amount,
    pub tx_in: TxIn,
}

fn make_keypair() -> (SecretKey, PublicKey) {
    let sk = SecretKey::new(&mut thread_rng());
    let pk = PublicKey::from_private_key(
        &SECP256K1,
        &PrivateKey {
            compressed: true,
            network: Network::Regtest,
            key: sk,
        },
    );

    (sk, pk)
}
