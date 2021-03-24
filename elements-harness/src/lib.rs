#![warn(
    unused_extern_crates,
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::fallible_impl_from,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::dbg_macro
)]
#![forbid(unsafe_code)]

pub mod elementd_rpc;
pub mod image;

use crate::image::ElementsCore;
use reqwest::Url;
use testcontainers::{clients, core::Port, Container, Docker};

pub use crate::elementd_rpc::Client;

pub type Result<T> = std::result::Result<T, Error>;

const ELEMENTSD_RPC_PORT: u16 = 18443;

#[derive(Debug)]
pub struct Elementsd<'c> {
    pub container: Container<'c, clients::Cli, ElementsCore>,
    pub node_url: Url,
}

impl<'c> Elementsd<'c> {
    /// Starts a new regtest elementsd container
    pub fn new(client: &'c clients::Cli, tag: &str) -> Result<Self> {
        let container = client.run(
            ElementsCore::default()
                .with_tag(tag)
                .with_mapped_port(Port {
                    local: 0,
                    internal: ELEMENTSD_RPC_PORT,
                }),
        );
        let port = container
            .get_host_port(ELEMENTSD_RPC_PORT)
            .ok_or(Error::PortNotExposed(ELEMENTSD_RPC_PORT))?;

        let auth = container.image().auth();
        let url = format!(
            "http://{}:{}@localhost:{}",
            &auth.username,
            dbg!(&auth.password),
            port
        );
        let url = Url::parse(&url)?;

        Ok(Self {
            container,
            node_url: url,
        })
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum Error {
    #[error("Url Parsing: ")]
    UrlParseError(#[from] url::ParseError),
    #[error("Docker port not exposed: ")]
    PortNotExposed(u16),
}
