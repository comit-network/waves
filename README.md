# Project Waves

Building Defi for Bitcoin on Liquid.

## What you will find here

This project includes:

- `bobtimus`: a daemon which acts as an automated market-maker offering buy and sell `L-BTC/L-USDt` trades based on a rate pulled from `Kraken`. It also serves a website which acts as an interface for browser extensions to reach `bobtimus`' HTTP API.
- `waves_wallet`: a Liquid wallet as a browser extension. When visiting `bobtimus`' website, a `waves_wallet` user can perform `L-BTC/L-USDt` atomic swaps in a couple of clicks.

## Try it out on _regtest_

_Requires Rust, Yarn, Docker and Nigiri._

1. `./waves-scripts start_extension_dev_env`.

This starts up an Elements node, a block explorer, `bobtimus` and opens a Firefox instance with `waves_wallet` pre-installed.

## What's coming

With atomic swaps serving as the foundation, financial products such as borrowing and lending are on the horizon.

## Where to reach us

If you have any questions about this project or want to keep up with its development, join the [COMIT-Liquid Matrix channel](https://matrix.to/#/#comit-liquid:matrix.org).
