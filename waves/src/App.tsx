import { Box, Button, Center, Flex, IconButton, Text, VStack } from "@chakra-ui/react";
import React, { MouseEvent, useEffect, useReducer } from "react";
import { BrowserRouter, Link, Route, Switch, useHistory } from "react-router-dom";
import { RingLoader } from "react-spinners";
import "./App.css";
import AssetSelector from "./components/AssetSelector";
import ExchangeIcon from "./components/ExchangeIcon";
import { useRateService } from "./hooks/RateService";
import SwapWithWallet from "./wallet/SwapWithWallet";
import UnlockWallet from "./wallet/UnlockWallet";

export enum AssetType {
    BTC = "BTC",
    USDT = "USDT",
}

export type AssetSide = "Alpha" | "Beta";

export type Action =
    | { type: "AlphaAmount"; value: number }
    | { type: "BetaAmount"; value: number }
    | { type: "AlphaAssetType"; value: AssetType }
    | { type: "BetaAssetType"; value: AssetType }
    | { type: "RateChange"; value: number }
    | { type: "SwapAssetTypes" };

interface State {
    alpha: AssetState;
    beta: AssetState;
    rate: number;
}

interface AssetState {
    type: AssetType;
    amount: number;
}

function reducer(state: State, action: Action) {
    switch (action.type) {
        case "BetaAmount":
            console.log(`Received new beta amount: ${action.value}`);
            return {
                beta: {
                    type: state.beta.type,
                    amount: state.rate,
                },
                alpha: {
                    type: state.alpha.type,
                    amount: action.value / state.rate,
                },
                rate: state.rate,
            };
        case "AlphaAmount":
            console.log(`Received new alpha amount: ${action.value}`);
            return {
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
        case "AlphaAssetType":
            console.log(`Received new alpha type: ${action.value}`);
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

        case "BetaAssetType":
            console.log(`Received new beta type: ${action.value}`);
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
        case "RateChange":
            // TODO: fix "set USDT to alpha, win!"-bug
            console.log(`Received a new rate: ${action.value}`);
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
        default:
            throw new Error("Unknown update action received");
    }
}

function App() {
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
    };

    const [state, dispatch] = useReducer(reducer, initialState);

    const [publishedTx, setPublishedTx] = React.useState("");
    const [txPending, setTxPending] = React.useState(false);

    const onConfirmed = (txId: string) => {
        console.log(`Transaction published ${txId}`);
        setTxPending(true);
        setPublishedTx(txId);

        setTimeout(() => {
            setTxPending(false);
        }, 3000);
    };

    const openBlockExplorer = (_clicked: MouseEvent) => {
        window.open(`https://blockstream.info/liquid/tx/${publishedTx}`, "_blank");
    };

    const rateService = useRateService();
    useEffect(() => {
        const subscription = rateService.subscribe((rate) => {
            // setBetaAmount(alphaAmount * rate); TODO update amount accordingly
            dispatch({
                type: "RateChange",
                value: rate,
            });
        });
        return () => {
            rateService.unsubscribe(subscription);
        };
    }, [rateService]);

    return (
        <div className="App">
            <header className="App-header">
                <BrowserRouter>
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
                                    <UnlockWallet />
                                </Route>
                                <Route path="/swap">
                                    <SwapWithWallet
                                        onConfirmed={onConfirmed}
                                        alphaAmount={state.alpha.amount}
                                        betaAmount={state.beta.amount}
                                        alphaAsset={state.alpha.type}
                                        betaAsset={state.beta.type}
                                    />
                                </Route>
                                <Route path="/done">
                                    <Button
                                        isLoading={txPending}
                                        size="lg"
                                        variant="main_button"
                                        spinner={<RingLoader size={50} color="white" />}
                                        onClick={openBlockExplorer}
                                    >
                                        Check Transaction
                                    </Button>
                                </Route>
                            </Switch>
                        </Box>
                    </VStack>
                </BrowserRouter>
            </header>
        </div>
    );
}

export default App;
