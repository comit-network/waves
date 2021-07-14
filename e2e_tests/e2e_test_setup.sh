set -e

nigiri start --liquid

sleep 5

btc_asset_id=$(nigiri rpc --liquid dumpassetlabels | jq -r '.bitcoin')

# We need to mine some blocks so that electrs API calls work
address=$(nigiri rpc --liquid getnewaddress)
nigiri rpc --liquid generatetoaddress 150 $address > /dev/null

response=$(nigiri rpc --liquid issueasset 10000000 1)
usdt_asset_id=$(echo $response | jq -r '.asset')

echo "USDT Asset ID: "$usdt_asset_id
echo "Bitcoin Asset ID: "$btc_asset_id

export $(cat ~/.nigiri/.env | xargs)

(
    cd ../extension
    yarn install

    export REACT_APP_CHAIN="ELEMENTS"
    export REACT_APP_ESPLORA_API_URL="http://localhost:$LIQUID_CHOPSTICKS_PORT"
    export REACT_APP_LBTC_ASSET_ID=$btc_asset_id
    export REACT_APP_LUSDT_ASSET_ID=$usdt_asset_id
    yarn build

    yarn package
)

(
    cd ../waves/

    yarn install
    export REACT_APP_BLOCKEXPLORER_URL="http://localhost:$LIQUID_ESPLORA_PORT"
    yarn run build

    cd ../

    cargo build --bin fake_bobtimus
    RUST_LOG=debug,hyper=info,reqwest=info cargo run --bin fake_bobtimus -- \
            start \
            --elementsd http://admin1:123@localhost:$LIQUID_NODE_PORT \
            --usdt $usdt_asset_id > e2e_tests/bobtimus.log 2>&1 &
)
