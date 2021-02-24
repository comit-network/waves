use crate::Component;
use serde::{Deserialize, Serialize};

/// Message to be send between in-page script and content script
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub data: String,
    pub target: Component,
    pub source: Component,
}
