import { Box, Button, Flex, Heading, Image, Spacer, Text } from "@chakra-ui/react";
import React from "react";
import { useAsync } from "react-async";
import { signAndSendSwap } from "../background-proxy";
import { LoanToSign, USDT_TICKER } from "../models";
import YouSwapItem from "./SwapItem";
import Usdt from "./tether.svg";

interface ConfirmLoanProps {
    onCancel: () => void;
    onSuccess: (txId: string) => void;
    loanToSign: LoanToSign;
}

export default function ConfirmLoan(
    { onCancel, onSuccess, loanToSign }: ConfirmLoanProps,
) {
    let { isPending, run } = useAsync({
        deferFn: async () => {
            const txId = await signAndSendSwap(loanToSign.txHex, loanToSign.tabId);
            onSuccess(txId);
        },
    });

    let { collateral, principal, principalRepayment, term } = loanToSign;

    return (<Box>
        <form
            onSubmit={async e => {
                e.preventDefault();
                run();
            }}
            data-cy="confirm-loan-form"
        >
            <Heading>Confirm Loan</Heading>
            <Box>
                <YouSwapItem
                    tradeSide={collateral}
                    action={"send"}
                />
            </Box>
            <Box>
                <YouSwapItem
                    tradeSide={principal}
                    action={"receive"}
                />
            </Box>
            <Box w="100%">
                <Flex>
                    <Box h="40px" p="1">
                        <Text>Repayment amount: {principalRepayment}</Text>
                    </Box>
                    <Spacer />
                    <Box w="40px" h="40px">
                        <Image src={Usdt} h="32px" />
                    </Box>
                    <Box h="40px" justify="right" p="1">
                        <Text align="center" justify="center">
                            {USDT_TICKER}
                        </Text>
                    </Box>
                </Flex>
            </Box>
            <Box w="100%">
                <Flex>
                    <Box h="40px" p="1">
                        <Text>Loan term: {term}</Text>
                    </Box>
                </Flex>
            </Box>

            <Button
                variant="secondary"
                mr={3}
                onClick={onCancel}
            >
                Cancel
            </Button>
            <Button
                type="submit"
                variant="primary"
                isLoading={isPending}
                data-cy="sign-and-send-button"
            >
                Sign and send
            </Button>
        </form>
    </Box>);
}
