import { Button, Center, useToast, VStack } from "@chakra-ui/react";
import React, { Dispatch } from "react";
import { Action, Asset, BorrowState, Rate } from "./App";
import calculateBetaAmount from "./calculateBetaAmount";
import NumberInput from "./components/NumberInput";
import RateInfo from "./components/RateInfo";

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
                    onClick={() => {
                        toast({
                            title: "Demo",
                            description: "This is currently just a mockup.",
                            status: "warning",
                            duration: 9000,
                            isClosable: true,
                        });
                    }}
                >
                    Take loan
                </Button>
            </Center>
        </VStack>
    );
}

export default Borrow;
