use serde::{Deserialize, Serialize};

/// Message to be send between content script and background script
#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub data: String,
    pub target: String,
    pub source: String,
}
