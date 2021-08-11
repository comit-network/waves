import { Box, Button, ButtonGroup, Center, Divider, HStack, Text, Tooltip, useToast, VStack } from "@chakra-ui/react";
import Debug from "debug";
import React, { Dispatch } from "react";
import { AsyncState, useAsync } from "react-async";
import { useHistory } from "react-router-dom";
import { Action, Asset, BorrowState, Rate } from "./App";
import { getLoanOffer, postLoanFinalization, postLoanRequest } from "./Bobtimus";
import calculateBetaAmount from "./calculateBetaAmount";
import NumberInput from "./components/NumberInput";
import RateInfo from "./components/RateInfo";
import WavesProvider from "./waves-provider";
import { Status, WalletStatus } from "./waves-provider/wavesProvider";

const debug = Debug("Borrow");
const error = Debug("Borrow:error");

interface BorrowProps {
    dispatch: Dispatch<Action>;
    rate: Rate;
    state: BorrowState;
    walletStatusAsyncState: AsyncState<WalletStatus>;
    wavesProvider: WavesProvider | undefined;
}

function Borrow({ dispatch, state, rate, wavesProvider, walletStatusAsyncState }: BorrowProps) {
    const toast = useToast();
    const history = useHistory();

    const loanOfferHook = useAsync({
        promiseFn: getLoanOffer,
        onResolve: (data) => {
            dispatch({
                type: "UpdateLoanOffer",
                value: data,
            });
        },
    });
    let { isLoading: loanOfferLoading, data: loanOffer } = loanOfferHook;
    // TODO: Some mechanism to refresh the loan offer values
    //  Note: The rate is currently not taken from the offer, but from the state's rate

    let { data: walletStatus, reload: reloadWalletStatus, error: walletStatusError } = walletStatusAsyncState;

    function onPrincipalAmountChange(newAmount: string) {
        dispatch({
            type: "UpdatePrincipalAmount",
            value: newAmount,
        });
    }

    const interestRate = loanOffer ? loanOffer.base_interest_rate : 0;
    const minPrincipal = loanOffer ? loanOffer.min_principal : 0;
    const maxPrincipal = loanOffer ? loanOffer.max_principal : 0;

    const principalAmount = Number.parseFloat(state.principalAmount);

    // TODO: Let the user define the collateral amount that is within Bobtimus' LTV bounds
    let collateralAmount = calculateBetaAmount(
        Asset.USDT,
        principalAmount,
        rate,
    );

    let interestAmount = principalAmount * interestRate;

    let { run: takeLoan, isLoading: isTakingLoan } = useAsync({
        deferFn: async () => {
            if (!wavesProvider) {
                error("Cannot borrow. Waves provider not found.");
                return;
            }

            try {
                const feeRate = state.loanOffer!.fee_sats_per_vbyte;

                let loanRequestWalletParams = await wavesProvider.makeLoanRequestPayload(
                    collateralAmount.toString(),
                    feeRate.toString(),
                );

                let loanResponse = await postLoanRequest(loanRequestWalletParams, state.loanTermInDays);
                debug(JSON.stringify(loanResponse));

                let loanTransaction = await wavesProvider.signLoan(loanResponse);
                let txid = await postLoanFinalization(loanTransaction);

                // TODO: Add different page for loaned?
                history.push(`/trade/swapped/${txid}`);
            } catch (e) {
                const description = typeof e === "string" ? e : JSON.stringify(e);

                toast({
                    title: "Error",
                    description,
                    status: "error",
                    duration: 5000,
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

    let loanButton;
    if (!wavesProvider || walletStatusError) {
        if (walletStatusError) {
            error(walletStatusError);
        }
        loanButton = <Button
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
                loanButton = <Button
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
                loanButton = <Button
                    variant="primary"
                    w="15rem"
                    isLoading={isTakingLoan}
                    onClick={takeLoan}
                    data-cy="data-cy-take-loan-button"
                >
                    Take loan
                </Button>;
                break;
        }
    }

    return (
        <VStack spacing={4} align="stretch">
            <Center bg="gray.100" w={800} h={320} borderRadius={"md"}>
                <HStack spacing={4}>
                    <VStack spacing={4}>
                        <Text align={"left"}>I want to borrow</Text>
                        <Tooltip
                            label={"min = " + minPrincipal + " max = " + maxPrincipal}
                            aria-label="principal"
                            hasArrow
                            placement={"right"}
                        >
                            <Box>
                                <NumberInput
                                    currency="$"
                                    value={state.principalAmount}
                                    precision={2}
                                    step={0.01}
                                    onAmountChange={onPrincipalAmountChange}
                                    isDisabled={loanOfferLoading}
                                    dataCy={"data-cy-principal"}
                                />
                            </Box>
                        </Tooltip>
                        <Text align={"left"}>Loan Term</Text>
                        <ButtonGroup variant="outline" colorScheme={"blue"} spacing="6">
                            <Button width={"100px"} variant={"solid"}>30 days</Button>
                            <Button width={"100px"}>90 days</Button>
                            <Button width={"100px"}>180 days</Button>
                        </ButtonGroup>
                        <Box>Collateralization</Box>
                        <ButtonGroup variant="outline" colorScheme={"blue"} spacing="6">
                            <Button width={"100px"}>130%</Button>
                            <Button width={"100px"} variant={"solid"}>150%</Button>
                            <Button width={"100px"}>200%</Button>
                        </ButtonGroup>
                    </VStack>
                    <Divider orientation={"vertical"} />
                    <VStack
                        spacing={4}
                        bg={"white"}
                        borderRadius={"md"}
                        pt={"5px"}
                        pb="25px"
                        pr="25px"
                        pl={"25px"}
                        boxShadow="xs"
                    >
                        <p>Collateral:</p>
                        <NumberInput
                            currency="₿"
                            value={collateralAmount}
                            precision={7}
                            step={0.000001}
                            onAmountChange={() => {}}
                            isDisabled={true}
                            dataCy={"data-cy-collateral"}
                        />
                        <p>Interest {interestRate * 100}%:</p>
                        <NumberInput
                            currency="$"
                            value={interestAmount}
                            precision={7}
                            step={0.01}
                            onAmountChange={() => {}}
                            isDisabled={true}
                            dataCy={"data-cy-interest"}
                        />
                    </VStack>
                </HStack>
            </Center>

            <RateInfo rate={rate} direction={"ask"} />

            <Center>
                {loanButton}
            </Center>
        </VStack>
    );
}

export default Borrow;
