use std::future::Future;

use anyhow::{bail, Context, Result};
use elements::encode::Encodable;
use elements::opcodes::all::*;
use elements::script::Builder;
use elements::secp256k1::rand::thread_rng;
use elements::secp256k1::{SecretKey, SECP256K1};
use elements::{bitcoin::util::psbt::serialize::Serialize, confidential::Nonce};
use elements::{
    bitcoin::{Amount, Network, PrivateKey, PublicKey},
    TxOutWitness,
};
use elements::{
    confidential::{Asset, Value},
    AddressParams,
};
use elements::{Address, AssetId, Script, Transaction, TxIn, TxOut};

#[cfg(test)]
mod happy_test;

pub struct LoanRequest {
    pub collateral_amount: Amount,
    pub collateral_tx_ins: Vec<TxIn>,
    pub collateral_change_tx_out: TxOut,
    pub tx_fee: Amount,
    pub borrower_pk: PublicKey,
    pub timelock: u64,
    pub principal_address: Address,
}

pub struct LoanResponse {
    pub transaction: Transaction,
    pub lender_pk: PublicKey,
    pub lender_repayment_address: Address,
    pub timelock: u64,
}

pub struct Borrower0 {
    pub keypair: (SecretKey, PublicKey),
    pub principal_address: Address,
    pub collateral_amount: Amount,
    pub collateral_tx_ins: Vec<TxIn>,
    pub collateral_change_tx_out: TxOut,
    pub tx_fee: Amount,
    pub timelock: u64,
    pub bitcoin_asset_id: AssetId,
    pub usdt_asset_id: AssetId,
}

impl Borrower0 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        principal_address: Address,
        collateral_amount: Amount,
        collateral_inputs: Vec<Input>,
        change_address: Address,
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
            script_pubkey: change_address.script_pubkey(),
            witness: TxOutWitness::default(),
        };

        let collateral_tx_ins = collateral_inputs
            .iter()
            .map(|input| input.tx_in.clone())
            .collect();

        Ok(Self {
            keypair,
            principal_address,
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
            principal_address: self.principal_address.clone(),
        }
    }

    pub fn interpret(&self, loan_response: LoanResponse) -> Result<Borrower1> {
        let transaction = loan_response.transaction;

        let principal_tx_out = transaction
            .output
            .iter()
            .find(|out| {
                out.asset.explicit().unwrap() == self.usdt_asset_id
                    && out.script_pubkey == self.principal_address.script_pubkey()
            })
            .context("no principal txout")?;

        let collateral_script = loan_contract(
            self.keypair.1,
            loan_response.lender_pk,
            Amount::from_sat(principal_tx_out.value.explicit().unwrap()),
            &loan_response.lender_repayment_address,
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
                    && (&out.script_pubkey) == &collateral_script_pubkey
            })
            .context("no collateral txout")?;

        transaction
            .output
            .iter()
            .find(|out| out == &&self.collateral_change_tx_out)
            .context("no collateral change txout")?;

        Ok(Borrower1 {
            loan_transaction: transaction,
        })
    }
}

pub struct Borrower1 {
    loan_transaction: Transaction,
}

impl Borrower1 {
    pub async fn sign<S, F>(&self, signer: S) -> Result<Transaction>
    where
        S: FnOnce(Transaction) -> F,
        F: Future<Output = Result<Transaction>>,
    {
        signer(self.loan_transaction.clone()).await
    }
}

pub struct Lender0 {
    pub keypair: (SecretKey, PublicKey),
    pub principal_inputs: Vec<Input>,
    pub repayment_address: Address,
    pub change_address: Address,
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
        repayment_address: Address,
        change_address: Address,
    ) -> Self {
        let keypair = make_keypair();

        Self {
            bitcoin_asset_id,
            usdt_asset_id,
            keypair,
            repayment_address,
            change_address,
            principal_inputs,
        }
    }

    pub fn interpret(self, loan_request: LoanRequest) -> Lender1 {
        let principal_amount = Lender0::calc_principal_amount(&loan_request);

        let (_, lender_pk) = self.keypair;
        let collateral_script = loan_contract(
            loan_request.borrower_pk,
            lender_pk,
            principal_amount,
            &self.repayment_address,
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
            script_pubkey: loan_request.principal_address.script_pubkey(),
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
            script_pubkey: self.change_address.script_pubkey(),
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
            repayment_address: self.repayment_address,
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
    pub repayment_address: Address,
    pub timelock: u64,
    pub loan_transaction: Transaction,
}

impl Lender1 {
    pub fn loan_response(&self) -> LoanResponse {
        LoanResponse {
            transaction: self.loan_transaction.clone(),
            lender_pk: self.keypair.1,
            lender_repayment_address: self.repayment_address.clone(),
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
) -> Script {
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

    Builder::new()
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
        .into_script()
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
