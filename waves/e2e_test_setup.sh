set -e

docker-compose up -d

sleep 5

native_asset_id=$(docker exec liquid-e2e-test elements-cli -rpcport=18884 -rpcuser=admin1 -rpcpassword=123 dumpassetlabels | jq -r '.bitcoin')
echo "Native Asset ID: "$native_asset_id

response=$(docker exec liquid-e2e-test elements-cli -rpcport=18884 -rpcuser=admin1 -rpcpassword=123 issueasset 10000000 1)
usdt_asset_id=$(echo $response | jq -r '.asset')
echo "USDT Asset ID: "$usdt_asset_id

yarn install
export NATIVE_ASSET_ID=$native_asset_id
export USDT_ASSET_ID=$usdt_asset_id
export CHAIN="ELEMENTS"
export ESPLORA_API_URL="http://localhost:3012"
export REACT_APP_BLOCKEXPLORER_URL="http://localhost:5001"

yarn build
RUST_LOG=info,bobtimus=debug cargo build --bin fake_bobtimus

RUST_LOG=info,bobtimus=debug cargo run --bin fake_bobtimus -- \
        --elementsd http://admin1:123@127.0.0.1:7041 \
        --usdt $usdt_asset_id > bobtimus.log 2>&1 &
