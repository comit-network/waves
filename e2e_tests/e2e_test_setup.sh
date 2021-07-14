set -e

# This script is for CI purposes only. If you want to run the e2e tests locally make sure to setup your environment accordingly using:
#          curl https://travis.nigiri.network | bash
#          docker-compose -f ./docker-compose.yml up -d

btc_asset_id=$(docker exec nigiri_liquid_1 elements-cli -rpcport=18884 -rpcuser=admin1 -rpcpassword=123 dumpassetlabels | jq -r '.bitcoin')

# We need to mine some blocks so that electrs API calls work
address=$(docker exec nigiri_liquid_1 elements-cli -rpcport=18884 -rpcuser=admin1 -rpcpassword=123 getnewaddress)

echo "Got new address which will be used for the miner: $address"
docker exec nigiri_liquid_1 elements-cli -rpcport=18884 -rpcuser=admin1 -rpcpassword=123 generatetoaddress 150 $address > /dev/null

response=$(docker exec nigiri_liquid_1 elements-cli -rpcport=18884 -rpcuser=admin1 -rpcpassword=123 issueasset 10000000 1)
usdt_asset_id=$(echo $response | jq -r '.asset')

echo "USDT Asset ID: "$usdt_asset_id
echo "Bitcoin Asset ID: "$btc_asset_id

(
    cd ../extension
    yarn install

    export REACT_APP_CHAIN="ELEMENTS"
    export REACT_APP_ESPLORA_API_URL="http://localhost:3001"
    export REACT_APP_LBTC_ASSET_ID=$btc_asset_id
    export REACT_APP_LUSDT_ASSET_ID=$usdt_asset_id
    yarn build

    yarn package
)

(
    cd ../waves/

    yarn install
    # TODO: current setup does not come with block explorer

    export REACT_APP_BLOCKEXPLORER_URL="http://localhost:5001"
    yarn run build

    cd ../

    cargo build --bin fake_bobtimus
    RUST_LOG=debug,hyper=info,reqwest=info cargo run --bin fake_bobtimus -- \
            start \
            --elementsd http://admin1:123@localhost:18884 \
            --usdt $usdt_asset_id > e2e_tests/bobtimus.log 2>&1 &
)
