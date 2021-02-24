use serde::{Deserialize, Serialize};

/// Message to be send between background script and popup script
#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub data: String,
    pub target: String,
    pub source: String,
    pub content_tab_id: u32,
}
