import { Box, Button, Center, HStack, Image, useToast } from "@chakra-ui/react";
import Debug from "debug";
import React, { useEffect, useReducer } from "react";
import { useAsync } from "react-async";
import { useSSE } from "react-hooks-sse";
import { useHistory } from "react-router-dom";
import "./App.css";
import { fundAddress } from "./Bobtimus";
import COMIT from "./components/comit_logo_spellout_opacity_50.svg";
import Trade from "./Trade";
import { getNewAddress, getWalletStatus } from "./wasmProxy";

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

export interface State {
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

    let walletStatusAsyncState = useAsync({
        promiseFn: getWalletStatus,
    });

    let { reload: reloadWalletStatus } = walletStatusAsyncState;

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
            <Trade
                state={state}
                dispatch={dispatch}
                rate={rate}
                walletStatusAsyncState={walletStatusAsyncState}
            />
        </Box>
    );
}

export default App;
