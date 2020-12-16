import { ExternalLinkIcon } from "@chakra-ui/icons";
import { Box, Button, Center, Flex, Link, Text, VStack } from "@chakra-ui/react";
import React, { useEffect, useReducer } from "react";
import { useRef } from "react";
import { BrowserRouter, Link as RouterLink, Redirect, Route, Switch } from "react-router-dom";
import { RingLoader } from "react-spinners";
import "./App.css";
import AssetSelector from "./components/AssetSelector";
import ExchangeIcon from "./components/ExchangeIcon";
import CreateWallet from "./CreateWallet";
import { useRateService } from "./hooks/RateService";
import SwapWithWallet from "./SwapWithWallet";
import UnlockWallet from "./UnlockWallet";
import WalletInfo from "./WalletInfo";
import { getBalances, getWalletStatus } from "./wasmProxy";

export enum AssetType {
    BTC = "BTC",
    USDT = "USDT",
}

export type AssetSide = "Alpha" | "Beta";

export type Action =
    | { type: "UpdateAlphaAmount"; value: number }
    | { type: "UpdateAlphaAssetType"; value: AssetType }
    | { type: "UpdateBetaAssetType"; value: AssetType }
    | { type: "UpdateRate"; value: number }
    | { type: "SwapAssetTypes" }
    | { type: "PublishTransaction"; value: string }
    | { type: "UpdateWalletStatus"; value: WalletStatus }
    | { type: "UpdateBalance"; value: WalletBalance };

interface State {
    alpha: AssetState;
    beta: AssetState;
    rate: number;
    txId: string;
    wallet: Wallet;
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
    beta: {
        type: AssetType.USDT,
        amount: 191.34,
    },
    rate: 19133.74,
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
                beta: {
                    type: state.beta.type,
                    amount: action.value * state.rate,
                },
                rate: state.rate,
            };
        case "UpdateAlphaAssetType":
            let beta = state.beta;
            if (beta.type === action.value) {
                beta.type = state.alpha.type;
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
                alpha.type = state.beta.type;
            }
            return {
                ...state,
                alpha: alpha,
                beta: {
                    type: action.value,
                    amount: state.beta.amount,
                },
            };
        case "UpdateRate":
            // TODO: fix "set USDT to alpha, win!"-bug
            return {
                ...state,
                beta: {
                    ...state.beta,
                    amount: state.alpha.amount * action.value,
                },
                rate: action.value,
            };
        case "SwapAssetTypes":
            return {
                ...state,
                alpha: state.beta,
                beta: state.alpha,
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

    const rateService = useRateService();
    useEffect(() => {
        const subscription = rateService.subscribe((rate) => {
            // setBetaAmount(alphaAmount * rate); TODO update amount accordingly
            dispatch({
                type: "UpdateRate",
                value: rate,
            });
        });
        return () => {
            rateService.unsubscribe(subscription);
        };
    }, [rateService]);

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
        });
    }, []);

    let walletUpdating = useRef(false);
    useEffect(() => {
        if (!walletUpdating.current) {
            walletUpdating.current = true;
            let updateBalanceInterval = setInterval(() => {
                if (state.wallet.status.loaded) {
                    getBalances().then((balances) => {
                        console.log(`Updated balances: ${balances}`);
                        dispatch({
                            type: "UpdateBalance",
                            value: {
                                btcBalance: 0,
                                usdtBalance: 0,
                            },
                        });
                    });
                }
            }, 5000);
            return () => {
                walletUpdating.current = false;
                clearTimeout(updateBalanceInterval);
            };
        }
    }, [walletUpdating, state.wallet.status.loaded]);

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
                                    <ExchangeIcon dispatch={dispatch} />
                                </Box>
                            </Center>
                            <AssetSelector
                                assetSide="Beta"
                                placement="right"
                                amount={state.beta.amount}
                                type={state.beta.type}
                                dispatch={dispatch}
                            />
                        </Flex>
                        <Box>
                            <Text textStyle="info">1 BTC = {state.rate} USDT</Text>
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
                                                    betaAmount={state.beta.amount}
                                                    alphaAsset={state.alpha.type}
                                                    betaAsset={state.beta.type}
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
