import { Button, Center, useToast, VStack } from "@chakra-ui/react";
import Debug from "debug";
import React, { Dispatch } from "react";
import { AsyncState, useAsync } from "react-async";
import { useHistory } from "react-router-dom";
import { Action, Asset, BorrowState, Rate } from "./App";
import { postLoanFinalization, postLoanRequest } from "./Bobtimus";
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

    let { data: walletStatus, reload: reloadWalletStatus, error: walletStatusError } = walletStatusAsyncState;

    // TODO: We should get an up-to-date interest rate from Bobtimus
    let interestRate = 0.10;

    const principalAmount = Number.parseFloat(state.principalAmount);
    let collateralAmount = calculateBetaAmount(
        Asset.USDT,
        principalAmount,
        rate,
    );

    let interestAmount = principalAmount * interestRate;

    function onPrincipalAmountChange(newAmount: string) {
        dispatch({
            type: "UpdatePrincipalAmount",
            value: newAmount,
        });
    }

    let { run: requestNewLoan, isLoading: isRequestingNewLoan } = useAsync({
        deferFn: async () => {
            if (!wavesProvider) {
                error("Cannot borrow. Waves provider not found.");
                return;
            }

            try {
                let loanRequest = await wavesProvider.makeLoanRequestPayload(collateralAmount.toString());
                let loanResponse = await postLoanRequest(loanRequest);
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
                    isLoading={isRequestingNewLoan}
                    onClick={requestNewLoan}
                >
                    Take loan
                </Button>;
                break;
        }
    }

    return (
        <VStack spacing={4} align="stretch">
            <Center bg="gray.100" w={400} h={400} borderRadius={"md"}>
                <VStack spacing={4}>
                    <p>Principal:</p>
                    <NumberInput
                        currency="$"
                        value={state.principalAmount}
                        precision={2}
                        step={0.01}
                        onAmountChange={onPrincipalAmountChange}
                        isDisabled={false}
                        dataCy={"principal"}
                    />
                    <p>Collateral:</p>
                    <NumberInput
                        currency="₿"
                        value={collateralAmount}
                        precision={7}
                        step={0.000001}
                        onAmountChange={() => {}}
                        isDisabled={true}
                        dataCy={"collateral"}
                    />
                    <p>Interest {interestRate}%:</p>
                    <NumberInput
                        currency="₿"
                        value={interestAmount}
                        precision={7}
                        step={0.01}
                        onAmountChange={() => {}}
                        isDisabled={true}
                        dataCy={"interest"}
                    />
                    <p>Loan term (in days): {state.loanTerm}</p>
                </VStack>
            </Center>

            <RateInfo rate={rate} direction={"ask"} />

            <Center>
                {loanButton}
            </Center>
        </VStack>
    );
}

export default Borrow;
