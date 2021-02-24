use serde::{Deserialize, Serialize};

pub mod bs_ps;
pub mod cs_bs;
pub mod ips_cs;

#[derive(Debug, Serialize, Deserialize)]
pub enum Component {
    Background,
    Content,
    InPage,
    PopUp,
}
