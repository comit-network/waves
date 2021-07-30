set -e

usage() { echo "Usage:
    To clean up old state run: ./e2e_test_setup -C
    To start docker containers run: ./e2e_test_setup -S
" 1>&2; exit 1; }

while getopts "CS" o; do
    case "${o}" in
        C)
            if [ -d "nigiri" ]
            then
                echo "Clearing docker compose data"
                cd nigiri
                docker compose down
                rm -rf liquid-config/liquidregtest
                cd ..
            else
                echo "Nigiri data does not exist."
            fi
            ;;
        S)
            echo "Starting docker containers"
            docker compose -f ./nigiri/docker-compose.yml up -d
            ;;
        *)
            usage
            ;;
    esac
done


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

    cargo build --bin bobtimus --features faucet
    RUST_LOG=debug,hyper=info,reqwest=info cargo run --bin bobtimus --features faucet -- \
            start \
            --http 127.0.0.1:3030 \
            --elementsd http://admin1:123@localhost:18884 \
            --usdt $usdt_asset_id > e2e_tests/bobtimus.log 2>&1 &
)
