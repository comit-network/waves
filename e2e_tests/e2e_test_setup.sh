set -e

# This script is primarily for CI purposes. If you want to run the e2e tests locally use the -L flag:
#    ./e2e_test_setup.sh -L

if getopts ":L" arg; then
  mkdir -p nigiri
  cd nigiri
  if [[ ! ( -d "config" ) || ! ( -d "liquid-config" ) || ! ( -f "docker-compose.yml" ) ]]; then
    curl https://travis.nigiri.network | bash
  fi
  docker-compose -f ./docker-compose.yml up -d
  cd ../
fi

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
            --http 127.0.0.1:3030 \
            --elementsd http://admin1:123@localhost:18884 \
            --usdt $usdt_asset_id > e2e_tests/bobtimus.log 2>&1 &
)
