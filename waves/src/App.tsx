import { ExternalLinkIcon } from "@chakra-ui/icons";
import { Box, Button, Center, Flex, Link, Text, useInterval, VStack } from "@chakra-ui/react";
import React, { useEffect, useReducer } from "react";
import { useSSE } from "react-hooks-sse";
import { BrowserRouter, Link as RouterLink, Redirect, Route, Switch } from "react-router-dom";
import { RingLoader } from "react-spinners";
import "./App.css";
import AssetSelector from "./components/AssetSelector";
import ExchangeIcon from "./components/ExchangeIcon";
import CreateWallet from "./CreateWallet";
import { calculateBetaAmount } from "./RateService";
import SwapWithWallet from "./SwapWithWallet";
import UnlockWallet from "./UnlockWallet";
import WalletInfo from "./WalletInfo";
import { getBalances, getWalletStatus } from "./wasmProxy";

export const LBTC_TICKER = "L-BTC";
export const LUSDT_TICKER = "USDt";

export enum AssetType {
    BTC = "BTC",
    USDT = "USDT",
}

export type AssetSide = "Alpha" | "Beta";

export type Action =
    | { type: "UpdateAlphaAmount"; value: number }
    | { type: "UpdateAlphaAssetType"; value: AssetType }
    | { type: "UpdateBetaAssetType"; value: AssetType }
    | {
        type: "SwapAssetTypes";
        value: {
            betaAmount: number;
        };
    }
    | { type: "PublishTransaction"; value: string }
    | { type: "UpdateWalletStatus"; value: WalletStatus }
    | { type: "UpdateBalance"; value: WalletBalance };

interface State {
    alpha: AssetState;
    beta: AssetType;
    txId: string;
    wallet: Wallet;
}

export interface Rate {
    ask: number;
    bid: number;
}

interface Wallet {
    balance: WalletBalance;
    status: WalletStatus;
}

interface WalletStatus {
    exists: boolean;
    loaded: boolean;
}

export interface WalletBalance {
    usdtBalance: number;
    btcBalance: number;
}

interface AssetState {
    type: AssetType;
    amount: number;
}

const initialState = {
    alpha: {
        type: AssetType.BTC,
        amount: 0.01,
    },
    beta: AssetType.USDT,
    rate: {
        ask: 19133.74,
        bid: 19133.74,
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
                    amount: action.value.betaAmount,
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
                        usdtBalance: action.value.usdtBalance,
                        btcBalance: action.value.btcBalance,
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
    const [state, dispatch] = useReducer(reducer, initialState);

    const [txPending, setTxPending] = React.useState(false);

    const onConfirmed = (txId: string) => {
        // TODO temp UI hack to make the button loading :)
        setTxPending(true);
        setTimeout(() => {
            setTxPending(false);
        }, 2000);
    };

    useEffect(() => {
        getWalletStatus().then((wallet_status) => {
            if (wallet_status.exists) {
                dispatch({
                    type: "UpdateWalletStatus",
                    value: {
                        exists: wallet_status.exists,
                        loaded: wallet_status.loaded,
                    },
                });
            } // by default `wallet.exists` is set to false, hence no need to handle
        }).catch((e) => {
            // TODO: handle error
        });
    }, []);

    useInterval(
        () => {
            getBalances().then((balances) => {
                console.log(`Updated balances: `, balances);
                let btcBalanceEntry = balances.find((balance) => balance.ticker === LBTC_TICKER);
                let usdtBalanceEntry = balances.find((balance) => balance.ticker === LUSDT_TICKER);

                const btcBalance = btcBalanceEntry ? btcBalanceEntry.value : 0;
                const usdtBalance = usdtBalanceEntry ? usdtBalanceEntry.value : 0;
                dispatch({
                    type: "UpdateBalance",
                    value: {
                        btcBalance,
                        usdtBalance,
                    },
                });
            });
        },
        state.wallet.status.loaded ? 5000 : null,
    );

    const rate = useSSE("rate", {
        ask: 19133.74,
        bid: 19133.74,
    });

    const betaAmount = calculateBetaAmount(state.alpha.type, state.alpha.amount, rate);

    return (
        <BrowserRouter>
            <div className="App">
                <header className="App-header">
                    <Route path="/swap">
                        <WalletInfo
                            balance={state.wallet.balance}
                        />
                    </Route>
                </header>
                <Center className="App-body">
                    <VStack
                        spacing={4}
                        align="stretch"
                    >
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
                        <Box>
                            <Text textStyle="info">1 BTC ~ {rate.ask} USDT</Text>
                        </Box>
                        <Box>
                            <Switch>
                                <Route exact path="/">
                                    {state.wallet.status.exists && <UnlockWallet dispatch={dispatch} />}
                                    {!state.wallet.status.exists && <CreateWallet dispatch={dispatch} />}
                                </Route>
                                <Route
                                    exact
                                    path="/swap"
                                    render={({ location }) =>
                                        state.wallet.status.loaded
                                            ? (
                                                <SwapWithWallet
                                                    onConfirmed={onConfirmed}
                                                    dispatch={dispatch}
                                                    alphaAmount={state.alpha.amount}
                                                    betaAmount={calculateBetaAmount(
                                                        state.alpha.type,
                                                        state.alpha.amount,
                                                        rate,
                                                    )}
                                                    alphaAsset={state.alpha.type}
                                                    betaAsset={state.beta}
                                                />
                                            )
                                            : (
                                                <Redirect
                                                    to={{
                                                        pathname: "/",
                                                        state: { from: location },
                                                    }}
                                                />
                                            )}
                                />
                                <Route exact path="/swap/done">
                                    <VStack>
                                        <Text textStyle="info">
                                            Check in <Link
                                                href={`https://blockstream.info/liquid/tx/${state.txId}`}
                                                isExternal
                                            >
                                                Block Explorer <ExternalLinkIcon mx="2px" />
                                            </Link>
                                        </Text>
                                        <Button
                                            isLoading={txPending}
                                            size="lg"
                                            variant="main_button"
                                            spinner={<RingLoader size={50} color="white" />}
                                            as={RouterLink}
                                            to="/swap"
                                        >
                                            Swap again?
                                        </Button>
                                    </VStack>
                                </Route>
                            </Switch>
                        </Box>
                    </VStack>
                </Center>
            </div>
        </BrowserRouter>
    );
}

export default App;
