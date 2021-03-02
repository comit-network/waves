use serde::{Deserialize, Serialize};

pub mod bs_ps;
pub mod cs_bs;
pub mod ips_cs;

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
