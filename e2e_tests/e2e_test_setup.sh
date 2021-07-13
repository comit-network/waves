set -e

docker-compose up -d

sleep 5

btc_asset_id=$(docker exec liquid elements-cli -rpcport=18884 -rpcuser=admin1 -rpcpassword=123 dumpassetlabels | jq -r '.bitcoin')

# We need to mine some blocks so that electrs API calls work
address=$(docker exec liquid elements-cli -rpcport=18884 -rpcuser=admin1 -rpcpassword=123 getnewaddress)
docker exec liquid elements-cli -rpcport=18884 -rpcuser=admin1 -rpcpassword=123 generatetoaddress 150 $address > /dev/null

response=$(docker exec liquid elements-cli -rpcport=18884 -rpcuser=admin1 -rpcpassword=123 issueasset 10000000 1)
usdt_asset_id=$(echo $response | jq -r '.asset')

echo "USDT Asset ID: "$usdt_asset_id
echo "Bitcoin Asset ID: "$btc_asset_id

(
    cd ../extension
    yarn install

    export REACT_APP_CHAIN="ELEMENTS"
    export REACT_APP_ESPLORA_API_URL="http://127.0.0.1:3012"
    export REACT_APP_LBTC_ASSET_ID=$btc_asset_id
    export REACT_APP_LUSDT_ASSET_ID=$usdt_asset_id
    yarn build

    yarn package
)

(
    cd ../waves/

    yarn install
    export REACT_APP_BLOCKEXPLORER_URL="http://localhost:5001"
    yarn run build

    cd ../

    cargo build --bin fake_bobtimus
    RUST_LOG=debug,hyper=info,reqwest=info cargo run --bin fake_bobtimus -- \
            start \
            --elementsd http://admin1:123@127.0.0.1:7041 \
            --usdt $usdt_asset_id > e2e_tests/bobtimus.log 2>&1 &
)
