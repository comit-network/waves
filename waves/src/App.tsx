import { ExternalLinkIcon } from "@chakra-ui/icons";
import { Box, Button, Center, Flex, HStack, Image, Link, Text, VStack } from "@chakra-ui/react";
import { useToast } from "@chakra-ui/react";
import Debug from "debug";
import React, { useEffect, useReducer } from "react";
import { useAsync } from "react-async";
import { useSSE } from "react-hooks-sse";
import { Route, Switch, useHistory, useParams } from "react-router-dom";
import "./App.css";
import { fundAddress, postBuyPayload, postSellPayload } from "./Bobtimus";
import calculateBetaAmount, { getDirection } from "./calculateBetaAmount";
import AssetSelector from "./components/AssetSelector";
import COMIT from "./components/comit_logo_spellout_opacity_50.svg";
import ExchangeIcon from "./components/ExchangeIcon";
import {
    getNewAddress,
    getWalletStatus,
    makeBuyCreateSwapPayload,
    makeSellCreateSwapPayload,
    signAndSend,
} from "./wasmProxy";

const debug = Debug("App");
const error = Debug("App:error");

export enum Asset {
    LBTC = "L-BTC",
    USDT = "USDt",
}

export type AssetSide = "Alpha" | "Beta";

export type Action =
    | { type: "UpdateAlphaAmount"; value: string }
    | { type: "UpdateAlphaAssetType"; value: Asset }
    | { type: "UpdateBetaAssetType"; value: Asset }
    | {
        type: "SwapAssetTypes";
        value: {
            betaAmount: number;
        };
    }
    | { type: "PublishTransaction"; value: string }
    | { type: "UpdateWalletStatus"; value: WalletStatus }
    | { type: "UpdateBalance"; value: Balances };

interface State {
    alpha: AssetState;
    beta: Asset;
    txId: string;
    wallet: Wallet;
}

export interface Rate {
    ask: number;
    bid: number;
}

interface Wallet {
    status: WalletStatus;
}

interface WalletStatus {
    exists: boolean;
    loaded: boolean;
}

export interface Balances {
    usdt: number;
    btc: number;
}

interface AssetState {
    type: Asset;
    amount: string;
}

const initialState = {
    alpha: {
        type: Asset.LBTC,
        amount: "0.01",
    },
    beta: Asset.USDT,
    rate: {
        ask: 33766.30,
        bid: 33670.10,
    },
    txId: "",
    wallet: {
        balance: {
            usdtBalance: 0,
            btcBalance: 0,
        },
        status: {
            exists: false,
            loaded: false,
        },
    },
};

export function reducer(state: State = initialState, action: Action) {
    switch (action.type) {
        case "UpdateAlphaAmount":
            return {
                ...state,
                alpha: {
                    type: state.alpha.type,
                    amount: action.value,
                },
            };
        case "UpdateAlphaAssetType":
            let beta = state.beta;
            if (beta === action.value) {
                beta = state.alpha.type;
            }
            return {
                ...state,
                beta,
                alpha: {
                    type: action.value,
                    amount: state.alpha.amount,
                },
            };

        case "UpdateBetaAssetType":
            let alpha = state.alpha;
            if (alpha.type === action.value) {
                alpha.type = state.beta;
            }
            return {
                ...state,
                alpha,
                beta: action.value,
            };
        case "SwapAssetTypes":
            return {
                ...state,
                alpha: {
                    type: state.beta,
                    amount: state.alpha.amount,
                },
                beta: state.alpha.type,
            };
        case "PublishTransaction":
            return {
                ...state,
            };
        case "UpdateBalance":
            return {
                ...state,
                wallet: {
                    ...state.wallet,
                    balance: {
                        usdtBalance: action.value.usdt,
                        btcBalance: action.value.btc,
                    },
                },
            };
        case "UpdateWalletStatus":
            return {
                ...state,
                wallet: {
                    ...state.wallet,
                    status: {
                        exists: action.value.exists,
                        loaded: action.value.loaded,
                    },
                },
            };
        default:
            throw new Error("Unknown update action received");
    }
}

function App() {
    const history = useHistory();
    const toast = useToast();
    const path = history.location.pathname;

    useEffect(() => {
        if (path === "/app") {
            history.replace("/");
        }
    }, [path, history]);

    const [state, dispatch] = useReducer(reducer, initialState);

    const rate = useSSE("rate", {
        ask: 33766.30,
        bid: 33670.10,
    });

    let { data: walletStatus, reload: reloadWalletStatus, error: walletStatusError } = useAsync({
        promiseFn: getWalletStatus,
    });

    useEffect(() => {
        let callback = (_message: MessageEvent) => {};
        // @ts-ignore
        if (!window.liquid) {
            callback = async (message: MessageEvent) => {
                debug("Received message: %s", message.data);
                await reloadWalletStatus();
            };
        }
        window.addEventListener("message", callback);

        return () => window.removeEventListener("message", callback);
    });

    let { run: makeNewSwap, isLoading: isCreatingNewSwap } = useAsync({
        deferFn: async () => {
            let payload;
            let tx;
            try {
                if (state.alpha.type === Asset.LBTC) {
                    payload = await makeSellCreateSwapPayload(state.alpha.amount.toString());
                    tx = await postSellPayload(payload);
                } else {
                    payload = await makeBuyCreateSwapPayload(state.alpha.amount.toString());
                    tx = await postBuyPayload(payload);
                }

                let txid = await signAndSend(tx);

                history.push(`/swapped/${txid}`);
            } catch (e) {
                let description: string;
                if (e.InsufficientFunds) {
                    // TODO: Include alpha asset type in message
                    description = `Insufficient funds in wallet: expected ${e.InsufficientFunds.needed},
                         got ${e.InsufficientFunds.available}`;
                } else if (e === "Rejected") {
                    description = "Swap not authorised by wallet extension";
                } else {
                    description = JSON.stringify(e);
                    error(e);
                }

                toast({
                    title: "Error",
                    description,
                    status: "error",
                    duration: 9000,
                    isClosable: true,
                });
            }
        },
    });

    const alphaAmount = Number.parseFloat(state.alpha.amount);
    const betaAmount = calculateBetaAmount(
        state.alpha.type,
        alphaAmount,
        rate,
    );

    async function unlock_wallet() {
        // TODO send request to open popup to unlock wallet
        debug("For now open popup manually...");
        await reloadWalletStatus();
    }

    async function get_extension() {
        // TODO forward to firefox app store
        debug("Download our awesome extension from...");
        await reloadWalletStatus();
    }

    let swapButton;
    if (walletStatusError) {
        error(walletStatusError);
        swapButton = <Button
            onClick={async () => {
                await get_extension();
            }}
            variant="primary"
            w="15rem"
            data-cy="get-extension-button"
        >
            Get Extension
        </Button>;
    } else if (walletStatus && (!walletStatus.exists || !walletStatus.loaded)) {
        swapButton = <Button
            onClick={async () => {
                await unlock_wallet();
            }}
            variant="primary"
            w="15rem"
            data-cy="unlock-wallet-button"
        >
            Unlock Wallet
        </Button>;
    } else {
        swapButton = <Button
            onClick={makeNewSwap}
            variant="primary"
            w="15rem"
            isLoading={isCreatingNewSwap}
            data-cy="swap-button"
        >
            Swap
        </Button>;
    }

    let { run: callFaucet, isLoading: isFaucetLoading } = useAsync({
        deferFn: async () => {
            try {
                let address = await getNewAddress();
                await fundAddress(address);
            } catch (e) {
                error("Could not call faucet: {}", e);
                toast({
                    title: "Error",
                    description: `Could not call faucet: ${e}`,
                    status: "error",
                    duration: 9000,
                    isClosable: true,
                });
            }
        },
    });

    return (
        <Box className="App">
            <header className="App-header">
                <HStack align="left">
                    <Button variant="secondary" onClick={callFaucet} isLoading={isFaucetLoading}>Faucet</Button>
                </HStack>
                <Center>
                    <Image src={COMIT} h="24px" />
                </Center>
            </header>
            <Center className="App-body">
                <Switch>
                    <Route exact path="/">
                        <VStack spacing={4} align="stretch">
                            <Flex color="white">
                                <AssetSelector
                                    assetSide="Alpha"
                                    placement="left"
                                    amount={state.alpha.amount}
                                    type={state.alpha.type}
                                    dispatch={dispatch}
                                />
                                <Center w="10px">
                                    <Box zIndex={2}>
                                        <ExchangeIcon
                                            onClick={() =>
                                                dispatch({
                                                    type: "SwapAssetTypes",
                                                    value: {
                                                        betaAmount,
                                                    },
                                                })}
                                            dataCy="exchange-asset-types-button"
                                        />
                                    </Box>
                                </Center>
                                <AssetSelector
                                    assetSide="Beta"
                                    placement="right"
                                    amount={betaAmount}
                                    type={state.beta}
                                    dispatch={dispatch}
                                />
                            </Flex>
                            <RateInfo rate={rate} direction={getDirection(state.alpha.type)} />
                            <Box>
                                {swapButton}
                            </Box>
                        </VStack>
                    </Route>

                    <Route exact path="/swapped/:txId">
                        <VStack>
                            <Text textStyle="smGray">
                                Check in{" "}
                                <BlockExplorerLink />
                            </Text>
                        </VStack>
                    </Route>
                </Switch>
            </Center>
        </Box>
    );
}

function BlockExplorerLink() {
    const { txId } = useParams<{ txId: string }>();
    const baseUrl = process.env.REACT_APP_BLOCKEXPLORER_URL
        ? `${process.env.REACT_APP_BLOCKEXPLORER_URL}`
        : "https://blockstream.info/liquid";

    return <Link
        href={`${baseUrl}/tx/${txId}`}
        isExternal
    >
        Block Explorer <ExternalLinkIcon mx="2px" />
    </Link>;
}

interface RateInfoProps {
    rate: Rate;
    direction: "ask" | "bid";
}

function RateInfo({ rate, direction }: RateInfoProps) {
    switch (direction) {
        case "ask":
            return <Box>
                <Text textStyle="smGray">{rate.ask} USDT ~ 1 BTC</Text>
            </Box>;
        case "bid":
            return <Box>
                <Text textStyle="smGray">1 BTC ~ {rate.bid} USDT</Text>
            </Box>;
    }
}

export default App;
