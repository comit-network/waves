use crate::Component;
use serde::{Deserialize, Serialize};

/// Message to be send between in-page script and content script
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub rpc_data: RpcData,
    pub target: Component,
    pub source: Component,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum RpcData {
    GetBalance,
    Balance,
    Hello(String),
}
