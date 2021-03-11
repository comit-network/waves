use elements::Txid;
use serde::{Deserialize, Serialize};
use wallet::{CreateSwapPayload, WalletStatus};

pub mod bs_ps;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Component {
    Background,
    Content,
    InPage,
    PopUp,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub target: Component,
    pub source: Component,
}

/// Requests sent from the in-page script to the background script.
#[derive(Debug, Deserialize, Serialize)]
pub enum ToBackground {
    StatusRequest,
    SellRequest(String),
    BuyRequest(String),
    SignRequest(String),
}

/// Responses sent from the background script to the in-page script.
#[derive(Debug, Deserialize, Serialize)]
pub enum ToPage {
    StatusResponse(WalletStatus),
    SellResponse(CreateSwapPayload),
    BuyResponse(CreateSwapPayload),
    SignResponse(Txid),
}
