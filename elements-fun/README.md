# elements-fun

Make it fun to work with the Elements blockchain!

This project is a fork (using git-subtree) of the [`elements`](https://github.com/ElementsProject/rust-elements) library.
We regularly pull in changes from upstream to keep up with its development.

## Differences to `rust-elements`.

- MSRV of Rust 1.29 is not enforced.
  As such we can make use of several features like `serde`'s derives, Rust 2018, etc.
- A focus on type-level programming.
  For example, we model transaction outputs as enums instead of individual fields being either _explicit_ or _confidential_.
- Direct integration with `libsecp256k1-zkp` for cryptographic aspects of Elements like blinding and unblinding assets.
  To achieve this, we forked `rust-secp256k1` [here](https://github.com/comit-network/rust-secp256k1-zkp).
  This library is currently not released on crates.io but we are planning on doing that eventually.
