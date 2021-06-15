import { Button, Center, useToast, VStack } from "@chakra-ui/react";
import Debug from "debug";
import React, { Dispatch } from "react";
import { useAsync } from "react-async";
import { Action, Asset, BorrowState, Rate } from "./App";
import { postLoanRequest } from "./Bobtimus";
import calculateBetaAmount from "./calculateBetaAmount";
import NumberInput from "./components/NumberInput";
import RateInfo from "./components/RateInfo";
import { makeLoanRequestPayload } from "./wasmProxy";

const debug = Debug("Borrow");
const error = Debug("Borrow:error");

interface BorrowProps {
    dispatch: Dispatch<Action>;
    rate: Rate;
    state: BorrowState;
}

function Borrow({ dispatch, state, rate }: BorrowProps) {
    const toast = useToast();

    // TODO: we should get interest rate from bobtimus
    let interestRate = 0.10;

    const amount = Number.parseFloat(state.principalAmount);
    let collateralAmount = calculateBetaAmount(
        Asset.USDT,
        amount,
        rate,
    );

    let interestAmount = amount * interestRate;

    function onCollateralAmountChange(newAmount: string) {
        dispatch({
            type: "UpdatePrincipalAmount",
            value: newAmount,
        });
    }

    let { run: requestNewLoan, isLoading: isRequestingNewLoan } = useAsync({
        deferFn: async () => {
            try {
                let payload = await makeLoanRequestPayload(state.principalAmount);
                let loanResponse = await postLoanRequest(payload);
                // TODO: forward response to wallet to sign and publish transaction
                // TODO: redirect to /trade/swapped/${txid}`

                debug(loanResponse);
            } catch (e) {
                let description = JSON.stringify(e);
                error(e);

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
                        onAmountChange={onCollateralAmountChange}
                        isDisabled={false}
                        data_cy={"principal"}
                    />
                    <p>Collateral:</p>
                    <NumberInput
                        currency="₿"
                        value={collateralAmount}
                        precision={7}
                        step={0.000001}
                        onAmountChange={() => {}}
                        isDisabled={true}
                        data_cy={"collateral"}
                    />
                    <p>Interest {interestRate}%:</p>
                    <NumberInput
                        currency="$"
                        value={interestAmount}
                        precision={7}
                        step={0.01}
                        onAmountChange={() => {}}
                        isDisabled={true}
                        data_cy={"collateral"}
                    />
                    <p>Loan term (in days): {state.loanTerm}</p>
                </VStack>
            </Center>

            <RateInfo rate={rate} direction={"ask"} />

            <Center>
                <Button
                    variant="primary"
                    w="15rem"
                    isLoading={isRequestingNewLoan}
                    onClick={requestNewLoan}
                >
                    Take loan
                </Button>
            </Center>
        </VStack>
    );
}

export default Borrow;
