# Waves wallet

## Customization

This wallet can be customized through the browser local storage (`window.localStorage`).
In particular, the library requires the following values to be there

- `ESPLORA_API_URL`: The base url to use for the Esplora instance of the Elements instance we are targeting.
- `BITCOIN_ASSET_ID`: The asset ID of the native asset of the Elements chain this wallet will be used on.
- `LUSDT_ASSET_ID`: The asset ID of the native asset of the Elements chain this wallet will be used on.
  Supported values are: `LIQUID` and `ELEMENTS`.
