use crate::Component;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Message to be send between background script and popup script
#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub rpc_data: RpcData,
    pub target: Component,
    pub source: Component,
    pub content_tab_id: u32,
}

// TODO: use proper types, this is just for ease of development
#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    pub value_map: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum RpcData {
    WalletStatus(String),
    UnlockWallet(Data),
    CreateWallet(String, String),
    Hello(String),
}

#[test]
fn test_serde() {
    let mut map = HashMap::with_capacity(2);
    map.insert("key1".to_string(), "value1".to_string());
    let msg = Message {
        rpc_data: RpcData::UnlockWallet(Data { value_map: map }),
        target: Component::Background,
        source: Component::PopUp,
        content_tab_id: 1,
    };

    let serialized = serde_json::to_string(&msg).unwrap();
    println!("serialized = {}", serialized);

    let msg_des: Message = serde_json::from_str(&serialized).unwrap();
    println!("deserialized = {:?}", msg_des);
}
