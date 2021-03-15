use serde::{Deserialize, Serialize};

/// Types used for communication between the in-page script and the background script.
///
/// These are also used by the content script, which acts as a middleman.
pub mod ips_bs {
    use super::*;
    use elements::Txid;
    use wallet::{CreateSwapPayload, WalletStatus};

    /// Requests sent from the in-page script to the background script.
    #[derive(Debug, Deserialize, Serialize)]
    pub enum ToBackground {
        WalletStatusRequest,
        SellRequest(String),
        BuyRequest(String),
        SignRequest(String),
    }

    /// Responses sent from the background script to the in-page script.
    #[derive(Debug, Deserialize, Serialize)]
    pub enum ToPage {
        StatusResponse(Result<WalletStatus, StatusError>),
        SellResponse(Result<CreateSwapPayload, MakePayloadError>),
        BuyResponse(Result<CreateSwapPayload, MakePayloadError>),
        SignResponse(Result<Txid, SignAndSendError>),
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct StatusError(pub String);

    #[derive(Debug, Deserialize, Serialize)]
    pub enum MakePayloadError {
        InsufficientFunds { needed: u64, available: u64 },
        Other(String),
    }

    impl From<wallet::MakePayloadError> for MakePayloadError {
        fn from(e: wallet::MakePayloadError) -> Self {
            match e {
                wallet::MakePayloadError::CoinSelection(
                    wallet::CoinSelectionError::InsufficientFunds { needed, available },
                ) => Self::InsufficientFunds { needed, available },
                e => Self::Other(format!("{:#}", e)),
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub enum SignAndSendError {
        ExtractTrade(String),
        Send(String),
        Rejected,
        Other(String),
    }

    impl From<wallet::SignAndSendError> for SignAndSendError {
        fn from(e: wallet::SignAndSendError) -> Self {
            match e {
                wallet::SignAndSendError::Send(e) => SignAndSendError::Send(e),
                e => Self::Other(format!("{:#}", e)),
            }
        }
    }
}

/// Types used for communication between the background script and the pop-up script.
pub mod bs_ps {
    use super::*;
    use wallet::{BalanceEntry, Trade};

    #[derive(Debug, Deserialize, Serialize)]
    /// Requests sent from the pop-up script to the background script.
    pub enum ToBackground {
        UnlockRequest(String, String),
        CreateWalletRequest(String, String),
        BackgroundStatusRequest,
        BalanceRequest,
        SignRequest { tx_hex: String, tab_id: u32 },
        Reject { tx_hex: String, tab_id: u32 },
        WithdrawAll(String),
    }

    #[derive(Debug, Deserialize, Serialize)]
    /// Responses sent from the background script to the pop-up script.
    pub enum ToPopup {
        BalanceResponse(Vec<BalanceEntry>),
        StatusResponse(BackgroundStatus),
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct BackgroundStatus {
        pub wallet: WalletStatus,
        pub sign_tx: Option<TransactionData>,
    }

    impl BackgroundStatus {
        pub fn new(wallet: WalletStatus, sign_tx: Option<TransactionData>) -> Self {
            Self { wallet, sign_tx }
        }
    }

    // TODO: Perhaps this state should be exposed by the `wallet`,
    // replacing the current `wallet::WalletStatus`. Having this type
    // in the messaging layer seems incorrect
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum WalletStatus {
        None,
        NotLoaded,
        Loaded { address: String },
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct TransactionData {
        pub hex: String,
        pub decoded: Trade,
        pub tab_id: u32,
    }
}
