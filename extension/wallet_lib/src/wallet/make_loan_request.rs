use crate::{
    constants::{NATIVE_ASSET_ID, USDT_ASSET_ID},
    storage::Storage,
    wallet::{current, get_txouts, Wallet},
};
use bdk::bitcoin::{Amount, Denomination};
use coin_selection::{self, coin_select};
use covenants::{Borrower0, LoanRequest};
use elements::{secp256k1_zkp::SECP256K1, OutPoint};
use estimate_transaction_size::avg_vbytes;
use futures::lock::Mutex;
use input::Input;
use rand::thread_rng;

pub async fn make_loan_request(
    name: String,
    current_wallet: &Mutex<Option<Wallet>>,
    collateral_amount: String,
) -> Result<LoanRequest, Error> {
    let collateral_amount = Amount::from_str_in(&collateral_amount, Denomination::Bitcoin)
        .map_err(|_| Error::ParseAmount(collateral_amount))?;

    let wallet = current(&name, current_wallet)
        .await
        .map_err(|_| Error::LoadWallet)?;
    let blinding_key = wallet.blinding_key();

    let coin_selector = {
        |amount, asset| async move {
            let wallet = current(&name, current_wallet)
                .await
                .map_err(|_| Error::LoadWallet)?;
            let utxos = get_txouts(&wallet, |utxo, txout| {
                Ok({
                    let unblinded_txout = txout.unblind(SECP256K1, blinding_key)?;
                    let outpoint = OutPoint {
                        txid: utxo.txid,
                        vout: utxo.vout,
                    };
                    let candidate_asset = unblinded_txout.asset;

                    if candidate_asset == asset {
                        Some((
                            coin_selection::Utxo {
                                outpoint,
                                value: unblinded_txout.value,
                                script_pubkey: txout.script_pubkey.clone(),
                                asset: candidate_asset,
                            },
                            txout,
                        ))
                    } else {
                        log::debug!(
                            "utxo {} with asset id {} is not the target asset, ignoring",
                            outpoint,
                            candidate_asset
                        );
                        None
                    }
                })
            })
            .await
            .map_err(Error::GetTxOuts)?;

            // Bob currently hardcodes a fee-rate of 1 sat / vbyte, hence
            // there is no need for us to perform fee estimation. Later
            // on, both parties should probably agree on a block-target
            // and use the same estimation service.
            let bobs_fee_rate = Amount::from_sat(1);
            let fee_offset = calculate_fee_offset(bobs_fee_rate);

            let output = coin_select(
                utxos.iter().map(|(utxo, _)| utxo).cloned().collect(),
                amount,
                bobs_fee_rate.as_sat() as f32,
                fee_offset,
            )
            .map_err(Error::CoinSelection)?;
            let selection = output
                .coins
                .iter()
                .map(|coin| {
                    let original_txout = utxos
                        .iter()
                        .find_map(|(utxo, txout)| (utxo.outpoint == coin.outpoint).then(|| txout))
                        .expect("same source of utxos")
                        .clone();

                    Input {
                        txin: coin.outpoint,
                        original_txout,
                        blinding_key,
                    }
                })
                .collect();

            Ok(selection)
        }
    };

    // TODO: Verify that the internal `anyhow::Error` is being mapped
    // to `make_loan_request::Error` correctly
    let borrower = Borrower0::new(
        &mut thread_rng(),
        coin_selector,
        wallet.get_address(),
        blinding_key,
        collateral_amount,
        // TODO: Make this dynamic once there is something going on on Liquid
        Amount::from_sat(1),
        //TODO: This must be chosen explicitly either by the borrower
        // through the UI or by Bobtimus via configuration
        0,
        *NATIVE_ASSET_ID,
        *USDT_ASSET_ID,
    )
    .await?;

    // TODO: Model separate errors for `borrower_state` storage
    let storage = Storage::local_storage().map_err(|_| Error::Storage)?;
    storage
        .set_item(
            "borrower_state",
            serde_json::to_string(&borrower).map_err(|_| Error::Storage)?,
        )
        .map_err(|_| Error::Storage)?;

    Ok(borrower.loan_request())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Amount string cannot be parsed: ")]
    ParseAmount(String),
    #[error("Wallet is not loaded")]
    LoadWallet,
    #[error("Coin selection: ")]
    CoinSelection(coin_selection::Error),
    #[error("Failed to get transaction outputs: ")]
    GetTxOuts(#[from] anyhow::Error),
    #[error("Failed to store borrower state")]
    Storage,
}

/// Calculate the fee offset required for the coin selection algorithm.
///
/// We are calculating this fee offset here so that we select enough coins to pay for the asset + the fee.
fn calculate_fee_offset(fee_sats_per_vbyte: Amount) -> Amount {
    let principal_outputs = 2; // one to pay the principal to the borrower and another as change for the lender
    let fee_offset = (principal_outputs * avg_vbytes::OUTPUT) * fee_sats_per_vbyte.as_sat();

    Amount::from_sat(fee_offset)
}
