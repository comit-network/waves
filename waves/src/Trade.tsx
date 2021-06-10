import { ExternalLinkIcon } from "@chakra-ui/icons";
import { Box, Button, Center, Flex, Link, StackDivider, Text, useToast, VStack } from "@chakra-ui/react";
import Debug from "debug";
import React, { Dispatch } from "react";
import { AsyncState, useAsync } from "react-async";
import { Route, Switch, useHistory, useParams } from "react-router-dom";
import { Action, Asset, Rate, TradeState } from "./App";
import { postBuyPayload, postSellPayload } from "./Bobtimus";
import calculateBetaAmount, { getDirection } from "./calculateBetaAmount";
import AssetSelector from "./components/AssetSelector";
import ExchangeIcon from "./components/ExchangeIcon";
import RateInfo from "./components/RateInfo";
import { makeBuyCreateSwapPayload, makeSellCreateSwapPayload, signAndSend, WalletStatus } from "./wasmProxy";

const debug = Debug("Swap");
const error = Debug("Swap:error");

interface SwapProps {
    state: TradeState;
    dispatch: Dispatch<Action>;
    rate: Rate;
    walletStatusAsyncState: AsyncState<WalletStatus>;
}

function Trade({ state, dispatch, rate, walletStatusAsyncState }: SwapProps) {
    const history = useHistory();
    const toast = useToast();

    const alphaAmount = Number.parseFloat(state.alpha.amount);
    const betaAmount = calculateBetaAmount(
        state.alpha.type,
        alphaAmount,
        rate,
    );

    let { data: walletStatus, reload: reloadWalletStatus, error: walletStatusError } = walletStatusAsyncState;

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

                history.push(`/trade/swapped/${txid}`);
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

    async function get_extension() {
        // TODO forward to firefox app store
        debug("Download our awesome extension from...");
        await reloadWalletStatus();
    }

    async function unlock_wallet() {
        // TODO send request to open popup to unlock wallet
        debug("For now open popup manually...");
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

    return (
        <Switch>
            <Route exact path="/trade">
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

            <Route exact path="/trade/swapped/:txId">
                <VStack>
                    <Text textStyle="smGray">
                        <VStack
                            divider={<StackDivider borderColor="gray.200" />}
                            spacing={4}
                            align="stretch"
                        >
                            <>
                                Check in{" "}
                                <BlockExplorerLink />
                            </>
                            <Button
                                variant="primary"
                                onClick={() => {
                                    history.push(`/trade`);
                                }}
                            >
                                Home
                            </Button>
                        </VStack>
                    </Text>
                </VStack>
            </Route>
        </Switch>
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

export default Trade;
