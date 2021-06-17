use serde::{Deserialize, Serialize};

/// Types used for communication between the in-page script and the background script.
///
/// These are also used by the content script, which acts as a middleman.
pub mod ips_bs {
    use super::*;
    use covenants::LoanRequest;
    use elements::{Address, Txid};
    use wallet::{CreateSwapPayload, WalletStatus};

    /// Requests sent from the in-page script to the background script.
    #[derive(Debug, Deserialize, Serialize)]
    pub enum ToBackground {
        WalletStatusRequest,
        SellRequest(String),
        BuyRequest(String),
        SignRequest(String),
        NewAddress,
        LoanRequest(String),
        SignLoan(String),
    }

    /// Responses sent from the background script to the in-page script.
    #[derive(Debug, Deserialize, Serialize)]
    pub enum ToPage {
        StatusResponse(Result<WalletStatus, StatusError>),
        SellResponse(Result<CreateSwapPayload, MakePayloadError>),
        BuyResponse(Result<CreateSwapPayload, MakePayloadError>),
        SignResponse(Result<Txid, SignAndSendError>),
        NewAddressResponse(Result<Address, NewAddressError>),
        LoanRequestResponse(Box<Result<LoanRequest, MakePayloadError>>),
        LoanTransaction(Result<String, SignLoanError>),
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
                    coin_selection::Error::InsufficientFunds { needed, available },
                ) => Self::InsufficientFunds { needed, available },
                e => Self::Other(format!("{:#}", e)),
            }
        }
    }

    impl From<wallet::MakeLoanRequestError> for MakePayloadError {
        fn from(e: wallet::MakeLoanRequestError) -> Self {
            match e {
                wallet::MakeLoanRequestError::CoinSelection(
                    coin_selection::Error::InsufficientFunds { needed, available },
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

    #[derive(Debug, Deserialize, Serialize)]
    pub enum SignLoanError {
        Rejected,
        Internal(String),
    }

    impl From<wallet::SignLoanError> for SignLoanError {
        fn from(e: wallet::SignLoanError) -> Self {
            Self::Internal(format!("{:#}", e))
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct NewAddressError(pub String);
}

/// Types used for communication between the background script and the pop-up script.
pub mod bs_ps {
    use super::*;
    use wallet::{BalanceEntry, LoanDetails, Trade};

    #[derive(Debug, Deserialize, Serialize)]
    /// Requests sent from the pop-up script to the background script.
    pub enum ToBackground {
        UnlockRequest(String, String),
        CreateWalletRequest(String, String),
        BackgroundStatusRequest,
        BalanceRequest,
        SignRequest { tx_hex: String, tab_id: u32 },
        Reject { tx_hex: String, tab_id: u32 },
        SignLoan { details: LoanDetails, tab_id: u32 },
        RejectLoan { details: LoanDetails, tab_id: u32 },
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
        pub sign_state: SignState,
    }

    impl BackgroundStatus {
        pub fn new(wallet: WalletStatus, sign_state: SignState) -> Self {
            Self { wallet, sign_state }
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub enum SignState {
        None,
        Trade(TransactionData),
        Loan(LoanData),
    }

    impl SignState {
        pub fn unset(&mut self) {
            *self = Self::None
        }
    }

    impl Default for SignState {
        fn default() -> Self {
            Self::None
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

    impl Default for WalletStatus {
        fn default() -> Self {
            Self::None
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct TransactionData {
        pub hex: String,
        pub decoded: Trade,
        pub tab_id: u32,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct LoanData {
        pub details: LoanDetails,
        pub tab_id: u32,
    }
}
