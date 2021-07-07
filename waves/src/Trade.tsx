import { ExternalLinkIcon } from "@chakra-ui/icons";
import { Box, Button, Center, Flex, HStack, Link, StackDivider, Text, useToast, VStack } from "@chakra-ui/react";
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
import WavesProvider from "./waves-provider";
import { Status, WalletStatus } from "./waves-provider/wavesProvider";

const debug = Debug("Swap");
const error = Debug("Swap:error");

interface SwapProps {
    state: TradeState;
    dispatch: Dispatch<Action>;
    rate: Rate;
    walletStatusAsyncState: AsyncState<WalletStatus>;
    wavesProvider: WavesProvider | undefined;
}

function Trade({ state, dispatch, rate, walletStatusAsyncState, wavesProvider }: SwapProps) {
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
            if (!wavesProvider) {
                error("Cannot swap. Waves provider not found.");
                return;
            }
            let tx;
            try {
                if (state.alpha.type === Asset.LBTC) {
                    const payload = await wavesProvider.getSellCreateSwapPayload(state.alpha.amount.toString());
                    tx = await postSellPayload(payload);
                } else {
                    const payload = await wavesProvider.getBuyCreateSwapPayload(state.alpha.amount.toString());
                    tx = await postBuyPayload(payload);
                }

                let txid = await wavesProvider.signAndSendSwap(tx);

                history.push(`/trade/swapped/${txid}`);
            } catch (e) {
                const description = typeof e === "string" ? e : JSON.stringify(e);

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
        reloadWalletStatus();
    }

    async function unlock_wallet() {
        // TODO send request to open popup to unlock wallet
        debug("For now open popup manually...");
        reloadWalletStatus();
    }

    let swapButton;

    if (!wavesProvider || walletStatusError) {
        if (walletStatusError) {
            error(walletStatusError);
        }

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
    } else {
        switch (walletStatus?.status) {
            case Status.None:
            case Status.NotLoaded:
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
                break;
            case Status.Loaded:
                swapButton = <Button
                    onClick={makeNewSwap}
                    variant="primary"
                    w="15rem"
                    isLoading={isCreatingNewSwap}
                    data-cy="swap-button"
                >
                    Swap
                </Button>;
                break;
        }
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
                    <VStack
                        divider={<StackDivider borderColor="gray.200" />}
                        spacing={4}
                        align="stretch"
                    >
                        <HStack>
                            <Box>
                                <Text textStyle="smGray">
                                    Check in{" "}
                                </Text>
                            </Box>
                            <Box>
                                <BlockExplorerLink />
                            </Box>
                        </HStack>
                        <Button
                            variant="primary"
                            onClick={() => {
                                history.push(`/trade`);
                            }}
                        >
                            Home
                        </Button>
                    </VStack>
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
