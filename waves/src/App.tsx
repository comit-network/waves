import { ExternalLinkIcon } from "@chakra-ui/icons";
import { Box, Button, Center, Flex, Image, Link, Text, useDisclosure, VStack } from "@chakra-ui/react";
import Debug from "debug";
import React, { useEffect, useReducer, useState } from "react";
import { useAsync } from "react-async";
import { useSSE } from "react-hooks-sse";
import { Route, Switch, useHistory, useParams } from "react-router-dom";
import useSWR from "swr";
import "./App.css";
import { postBuyPayload, postSellPayload } from "./Bobtimus";
import calculateBetaAmount, { getDirection } from "./calculateBetaAmount";
import AssetSelector from "./components/AssetSelector";
import COMIT from "./components/comit_logo_spellout_opacity_50.svg";
import ExchangeIcon from "./components/ExchangeIcon";
import ConfirmSwapDrawer from "./ConfirmSwapDrawer";
import CreateWalletDrawer from "./CreateWalletDrawer";
import UnlockWalletDrawer from "./UnlockWalletDrawer";
import WalletBalances from "./WalletBalances";
import WalletDrawer from "./WalletDrawer";
import {
    extractTrade,
    getBalances,
    getWalletStatus,
    makeBuyCreateSwapPayload,
    makeSellCreateSwapPayload,
    Trade,
} from "./wasmProxy";

const debug = Debug("App");

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
    const path = history.location.pathname;

    useEffect(() => {
        if (path === "/app") {
            history.replace("/");
        }
    }, [path, history]);

    const [[transaction, trade], setTransaction] = useState<[string, Trade]>(["", {} as any]);
    const [state, dispatch] = useReducer(reducer, initialState);

    const rate = useSSE("rate", {
        ask: 33766.30,
        bid: 33670.10,
    });

    let { isOpen: isUnlockWalletOpen, onClose: onUnlockWalletClose, onOpen: onUnlockWalletOpen } = useDisclosure();
    let { isOpen: isCreateWalletOpen, onClose: onCreateWalletClose, onOpen: onCreateWalletOpen } = useDisclosure();
    let { isOpen: isConfirmSwapOpen, onClose: onConfirmSwapClose, onOpen: onConfirmSwapOpen } = useDisclosure();
    let { isOpen: isWalletOpen, onClose: onWalletClose, onOpen: onWalletOpen } = useDisclosure();

    let { data: getWalletStatusResponse, isLoading, reload: reloadWalletStatus } = useAsync({
        promiseFn: getWalletStatus,
    });
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
        balance => balance.ticker === Asset.LBTC,
    );
    let usdtBalanceEntry = balances.find(
        balance => balance.ticker === Asset.USDT,
    );

    const btcBalance = btcBalanceEntry ? btcBalanceEntry.value : 0;
    const usdtBalance = usdtBalanceEntry ? usdtBalanceEntry.value : 0;

    let { run: makeNewSwap, isLoading: isCreatingNewSwap } = useAsync({
        deferFn: async () => {
            let payload;
            let tx;
            if (state.alpha.type === Asset.LBTC) {
                payload = await makeSellCreateSwapPayload(state.alpha.amount.toString());
                tx = await postSellPayload(payload);
            } else {
                payload = await makeBuyCreateSwapPayload(state.alpha.amount.toString());
                tx = await postBuyPayload(payload);
            }

            let trade = await extractTrade(tx);

            setTransaction([tx, trade]);

            onConfirmSwapOpen();
        },
    });

    const alphaAmount = Number.parseFloat(state.alpha.amount);
    const betaAmount = calculateBetaAmount(
        state.alpha.type,
        alphaAmount,
        rate,
    );

    let walletBalances;

    async function hello_backend_script() {
        // @ts-ignore
        if (!window.call_backend) {
            debug("Wallet provider not found");
        } else {
            debug("calling through to wallet");
            // @ts-ignore
            let message = await window.call_backend("Hello");
            debug(`PS: received from IPS: ${message}`);
        }
    }

    if (!walletStatus.exists) {
        walletBalances = <Button
            onClick={async () => {
                await hello_backend_script();
            }}
            size="sm"
            variant="primary"
            isLoading={isLoading}
            data-cy="create-wallet-button"
        >
            Create wallet
        </Button>;
    } else if (walletStatus.exists && !walletStatus.loaded) {
        walletBalances = <Button
            onClick={onUnlockWalletOpen}
            size="sm"
            variant="primary"
            isLoading={isLoading}
            data-cy="unlock-wallet-button"
        >
            Unlock wallet
        </Button>;
    } else {
        walletBalances = <WalletBalances
            balances={{
                usdt: usdtBalance,
                btc: btcBalance,
            }}
            onClick={onWalletOpen}
        />;
    }

    let isSwapButtonDisabled = state.alpha.type === Asset.LBTC
        ? btcBalance < alphaAmount
        : usdtBalance < alphaAmount;

    return (
        <Box className="App">
            <header className="App-header">
                {walletBalances}
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
                                <Button
                                    onClick={makeNewSwap}
                                    variant="primary"
                                    w="15rem"
                                    isLoading={isCreatingNewSwap}
                                    disabled={isSwapButtonDisabled}
                                    data-cy="swap-button"
                                >
                                    Swap
                                </Button>
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
            {isWalletOpen && <WalletDrawer
                balances={{
                    usdt: usdtBalance,
                    btc: btcBalance,
                }}
                onClose={onWalletClose}
                reloadBalances={async () => {
                    await reloadWalletBalances();
                }}
            />}
            {isUnlockWalletOpen && <UnlockWalletDrawer
                onCancel={onUnlockWalletClose}
                onUnlock={async () => {
                    await reloadWalletBalances();
                    await reloadWalletStatus();
                    onUnlockWalletClose();
                }}
            />}
            {isCreateWalletOpen && <CreateWalletDrawer
                onCancel={onCreateWalletClose}
                onCreate={async () => {
                    await reloadWalletStatus();
                    onCreateWalletClose();
                }}
            />}
            {isConfirmSwapOpen && <ConfirmSwapDrawer
                onCancel={onConfirmSwapClose}
                onSwapped={(txId) => {
                    history.push(`/swapped/${txId}`);
                    onConfirmSwapClose();
                }}
                transaction={transaction}
                trade={trade}
            />}

            <footer className="App-footer">
                <Center>
                    <Image src={COMIT} h="24px" />
                </Center>
            </footer>
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
