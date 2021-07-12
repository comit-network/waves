# Project Waves

Building Defi for Bitcoin on Liquid.

## What you will find here

This project includes:

- `bobtimus`: a daemon which acts as an automated market-maker offering buy and sell `L-BTC/L-USDt` trades based on a rate pulled from `Kraken`. It also serves a website which acts as an interface for browser extensions to reach `bobtimus`' HTTP API.
- `waves_wallet`: a Liquid wallet as a browser extension. When visiting `bobtimus`' website, a `waves_wallet` user can perform `L-BTC/L-USDt` atomic swaps in a couple of clicks.

## Try it out on _regtest_


# How to get started

Install nigiri from https://github.com/vulpemventures/nigiri

```bash
curl https://getnigiri.vulpem.com | bash
```

For our scripts you will need `jq` as well. You can get it from https://stedolan.github.io/jq/.

Start nodes:

```bash
./waves-scripts start_nodes
```

For our tools to work we need to know which asset ID is the native asset (i.e. Bitcoin).
```bash
./waves-scripts load_native_asset
```

This will create a file in your directory with an asset ID in it called `.native_asset_id`

If successful, we need to mint a new asset. This asset will be our USDT asset for the purpose of atomic swaps and lending.
The asset ID of this new asset will be in `.usdt_asset_id`

```bash
./waves-scripts mint_usdt
```

Afterwards we can go ahead and start bobtimus:

```bash
./waves-scripts start_bobtimus
```

This will start bobtimus in a detached mode. Its PID can be found in `.bobtimus` and the logs in `./logs/bobtimus`.

While bobtimus is hosting a production version of waves on `http://localhost:3030` you probably want a development
build while working on it.
For that run the following command and keep the terminal open. Your waves application will be reachable under
`http://localhost:3004`

```bash
./waves-script start_webapp
```

Last but not least, you will need the web extension.
We will need to go into the extension directory and run the following two commands:

```bash
cd extension;
yarn install # only needed once
yarn watch
```

This command will start `create-react-app` in watch mode and will automatically re-build the extension.

```bash
yarn start
```

This will start a firefox browser with the extension enabled and the browser console being opened.
The latter one is useful for degbugging purposes as it will print all kinds of stuff.

## What's coming

With atomic swaps serving as the foundation, financial products such as borrowing and lending are on the horizon.

## Where to reach us

If you have any questions about this project or want to keep up with its development, join the [COMIT-Liquid Matrix channel](https://matrix.to/#/#comit-liquid:matrix.org).

