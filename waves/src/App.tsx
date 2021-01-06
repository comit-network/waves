import { ExternalLinkIcon } from "@chakra-ui/icons";
import { Box, Button, Center, Flex, Link, Text, useDisclosure, VStack } from "@chakra-ui/react";
import React, { useReducer, useState } from "react";
import { useAsync } from "react-async";
import { useSSE } from "react-hooks-sse";
import { Route, Switch, useHistory, useParams } from "react-router-dom";
import useSWR from "swr";
import "./App.css";
import { postSellPayload } from "./Bobtimus";
import AssetSelector from "./components/AssetSelector";
import ExchangeIcon from "./components/ExchangeIcon";
import ConfirmSwapDrawer from "./ConfirmSwapDrawer";
import CreateWalletDrawer from "./CreateWalletDrawer";
import { calculateBetaAmount } from "./RateService";
import UnlockWalletDrawer from "./UnlockWalletDrawer";
import WalletInfo from "./WalletInfo";
import { getBalances, getWalletStatus, makeCreateSellSwapPayload } from "./wasmProxy";

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
    const history = useHistory();
    const [transaction, setTransaction] = useState("");
    const [state, dispatch] = useReducer(reducer, initialState);

    const rate = useSSE("rate", {
        ask: 19133.74,
        bid: 19133.74,
    });

    let { isOpen: isUnlockWalletOpen, onClose: onUnlockWalletClose, onOpen: onUnlockWalletOpen } = useDisclosure();
    let { isOpen: isCreateWalletOpen, onClose: onCreateWalletClose, onOpen: onCreateWalletOpen } = useDisclosure();
    let { isOpen: isConfirmSwapOpen, onClose: onConfirmSwapClose, onOpen: onConfirmSwapOpen } = useDisclosure();

    let { data: getWalletStatusResponse, isValidating: isLoading, mutate: reloadWalletStatus } = useSWR(
        "wallet-status",
        () => getWalletStatus(),
    );
    let walletStatus = getWalletStatusResponse || { exists: false, loaded: false };

    let { data: getBalancesResponse, mutate: reloadWalletBalances } = useSWR(
        () => walletStatus.loaded ? "wallet-balances" : null,
        () => getBalances(),
        {
            refreshInterval: 5000,
        },
    );
    let balances = getBalancesResponse || [];

    let btcBalanceEntry = balances.find(
        balance => balance.ticker === LBTC_TICKER,
    );
    let usdtBalanceEntry = balances.find(
        balance => balance.ticker === LUSDT_TICKER,
    );

    const btcBalance = btcBalanceEntry ? btcBalanceEntry.value : 0;
    const usdtBalance = usdtBalanceEntry ? usdtBalanceEntry.value : 0;

    let { run: makeNewSwap, isLoading: isCreatingNewSwap } = useAsync({
        deferFn: async () => {
            let payload = await makeCreateSellSwapPayload(state.alpha.amount.toString());
            let tx = await postSellPayload(payload);

            setTransaction(tx);

            onConfirmSwapOpen();
        },
    });

    const betaAmount = calculateBetaAmount(
        state.alpha.type,
        state.alpha.amount,
        rate,
    );

    let button;

    if (!walletStatus.exists) {
        button = <Button
            onClick={onCreateWalletOpen}
            size="lg"
            variant="main_button"
            isLoading={isLoading}
        >
            Create new wallet
        </Button>;
    } else if (walletStatus.exists && !walletStatus.loaded) {
        button = <Button
            onClick={onUnlockWalletOpen}
            size="lg"
            variant="main_button"
            isLoading={isLoading}
        >
            Unlock wallet
        </Button>;
    } else {
        button = <Button
            onClick={makeNewSwap}
            size="lg"
            variant="main_button"
            isLoading={isCreatingNewSwap}
        >
            Swap
        </Button>;
    }

    return (
        <div className="App">
            <header className="App-header">
                {walletStatus.loaded && <WalletInfo
                    balance={{
                        usdtBalance,
                        btcBalance,
                    }}
                />}
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
                                {button}
                            </Box>
                        </VStack>
                    </Route>

                    <Route exact path="/swapped/:txId">
                        <VStack>
                            <Text textStyle="info">
                                Check in{" "}
                                <BlockExplorerLink />
                            </Text>
                        </VStack>
                    </Route>
                </Switch>
            </Center>
            {/* TODO: Likely we only want this to be a single drawer because only one of them can be open at a time */}
            <UnlockWalletDrawer
                isOpen={isUnlockWalletOpen}
                onCancel={onUnlockWalletClose}
                onUnlock={async () => {
                    await reloadWalletBalances();
                    await reloadWalletStatus();
                    onUnlockWalletClose();
                }}
            />
            <CreateWalletDrawer
                isOpen={isCreateWalletOpen}
                onCancel={onCreateWalletClose}
                onCreate={async () => {
                    await reloadWalletStatus();
                    onCreateWalletClose();
                }}
            />
            <ConfirmSwapDrawer
                isOpen={isConfirmSwapOpen}
                onCancel={onConfirmSwapClose}
                transaction={transaction}
                onSwapped={(txId) => {
                    history.push(`/swapped/${txId}`);
                    onConfirmSwapClose();
                }}
            />
        </div>
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

export default App;
