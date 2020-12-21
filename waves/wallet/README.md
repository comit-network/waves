# Waves wallet

## Customization

This wallet can be customized through the following **build-time** environment variables:

- `ELEMENTS_ESPLORA_URL`: The base url to use for the Esplora instance of the Elements instance we are targeting.
  Defaults to `https://blockstream.info/liquid/api`.
- `NATIVE_ASSET_TICKER`: The ticker symbol of the native asset of the Elements chain this wallet will be used on.
  Defaults to `L-BTC`.
- `NATIVE_ASSET_ID`: The asset ID of the native asset of the Elements chain this wallet will be used on.
  Defaults to `6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d`.
- `ELEMENTS_CHAIN`: The name of the Elements chain we are targeting.
  Defaults to `LIQUID`.
  Supported values are: `LIQUID` and `ELEMENTS`.
