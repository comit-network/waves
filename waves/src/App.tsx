import { Box, Button, Center, HStack, Image, useToast, VStack } from "@chakra-ui/react";
import Debug from "debug";
import React, { useEffect, useReducer } from "react";
import { useAsync } from "react-async";
import { useSSE } from "react-hooks-sse";
import { Link as RouteLink, Redirect, Route, Switch, useHistory } from "react-router-dom";
import "./App.css";
import { fundAddress } from "./Bobtimus";
import Borrow from "./Borrow";
import COMIT from "./components/comit_logo_spellout_opacity_50.svg";
import Trade from "./Trade";

Debug.enable("*");
const debug = Debug("App");
const error = Debug("App:error");

export enum Asset {
    LBTC = "L-BTC",
    USDT = "USDt",
}

export type AssetSide = "Alpha" | "Beta";

export type Action =
    | { type: "UpdateAlphaAmount"; value: string }
    | { type: "UpdatePrincipalAmount"; value: string }
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

export interface TradeState {
    alpha: AssetState;
    beta: Asset;
    txId: string;
}

export interface BorrowState {
    loanTerm: number;
    principalAmount: string;
}

export interface State {
    trade: TradeState;
    borrow: BorrowState;
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
    trade: {
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
    },
    borrow: {
        principalAmount: "33766.3",
        loanTerm: 30,
    },
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
                trade: {
                    ...state.trade,
                    alpha: {
                        type: state.trade.alpha.type,
                        amount: action.value,
                    },
                },
            };
        case "UpdateAlphaAssetType":
            let beta = state.trade.beta;
            if (beta === action.value) {
                beta = state.trade.alpha.type;
            }
            return {
                ...state,
                trade: {
                    ...state.trade,
                    beta,
                    alpha: {
                        type: action.value,
                        amount: state.trade.alpha.amount,
                    },
                },
            };

        case "UpdateBetaAssetType":
            let alpha = state.trade.alpha;
            if (alpha.type === action.value) {
                alpha.type = state.trade.beta;
            }
            return {
                ...state,
                trade: {
                    ...state.trade,
                    alpha,
                    beta: action.value,
                },
            };
        case "SwapAssetTypes":
            return {
                ...state,
                trade: {
                    ...state.trade,
                    alpha: {
                        type: state.trade.beta,
                        amount: state.trade.alpha.amount,
                    },
                    beta: state.trade.alpha.type,
                },
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

        case "UpdatePrincipalAmount":
            return {
                ...state,
                borrow: {
                    ...state.borrow,
                    principalAmount: action.value,
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

    const wavesProvider = window.wavesProvider;

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
        promiseFn: wavesProvider?.walletStatus,
    });

    let { run: callFaucet, isLoading: isFaucetLoading } = useAsync({
        deferFn: async () => {
            try {
                if (wavesProvider) {
                    let address = await wavesProvider.getNewAddress();
                    await fundAddress(address);
                } else {
                    debug("Waves provider undefined. Cannot call faucet without an address");
                }
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
            <Route exact path="/">
                <Redirect to="/trade" />
            </Route>

            <header className="App-header">
                <HStack align="left">
                    <Button variant="secondary" onClick={callFaucet} isLoading={isFaucetLoading}>Faucet</Button>
                </HStack>
                <Center>
                    <Image src={COMIT} h="24px" />
                </Center>
            </header>

            <Center className="App-body">
                <VStack spacing={4}>
                    <HStack spacing={4} as="nav">
                        <NavLink text="Trade" path={"/trade"} />
                        <NavLink text="Borrow" path={"/borrow"} />
                    </HStack>
                    <Switch>
                        <Route path="/trade">
                            <Trade
                                state={state.trade}
                                dispatch={dispatch}
                                rate={rate}
                                walletStatusAsyncState={walletStatusAsyncState}
                                wavesProvider={wavesProvider}
                            />
                        </Route>
                        <Route path="/borrow">
                            <Borrow
                                dispatch={dispatch}
                                state={state.borrow}
                                rate={rate}
                                walletStatusAsyncState={walletStatusAsyncState}
                                wavesProvider={wavesProvider}
                            />
                        </Route>
                    </Switch>
                </VStack>
            </Center>
        </Box>
    );
}

type NavLinkProps = { text: string; path: string };
const NavLink = ({ text, path }: NavLinkProps) => (
    <RouteLink to={path}>
        <Route
            path={path}
            children={({ match }) => (
                <Button colorScheme="blue" variant={match?.path ? "solid" : "outline"}>
                    {text}
                </Button>
            )}
        />
    </RouteLink>
);

export default App;
