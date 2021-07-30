import { Box, Button, Flex, Heading, Image, Spacer, Text, useInterval } from "@chakra-ui/react";
import Debug from "debug";
import moment from "moment";
import React from "react";
import { useAsync } from "react-async";
import { getBlockHeight, signLoan } from "../background-proxy";
import { LoanToSign, USDT_TICKER } from "../models";
import YouSwapItem from "./SwapItem";
import Usdt from "./tether.svg";

const debug = Debug("confirmloan:error");

interface ConfirmLoanProps {
    onCancel: (tabId: number) => void;
    onSuccess: () => void;
    loanToSign: LoanToSign;
}

export default function ConfirmLoan(
    { onCancel, onSuccess, loanToSign }: ConfirmLoanProps,
) {
    let { isPending, run } = useAsync({
        deferFn: async () => {
            await signLoan(loanToSign.tabId);
            onSuccess();
        },
    });

    let { details: { collateral, principal, principalRepayment, term } } = loanToSign;

    const blockHeightHook = useAsync({
        promiseFn: getBlockHeight,
        onReject: (e) => debug("Failed to fetch block height %s", e),
    });
    let { data: blockHeight, reload: reloadBlockHeight } = blockHeightHook;

    useInterval(() => {
        reloadBlockHeight();
    }, 6000); // 1 min

    // format the time nicely into something like : `in 13 hours` or `in 1 month`.
    // block-height and loan-term are in "blocktime" ^= minutes
    const deadline = blockHeight && term
        ? moment().add(Math.abs(blockHeight - term), "minutes").fromNow()
        : null;

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
                        <Text>Loan term: {term} block-height {deadline ? "(due " + deadline + ")" : ""}</Text>
                    </Box>
                </Flex>
            </Box>

            <Button
                variant="secondary"
                mr={3}
                onClick={() => onCancel(loanToSign.tabId)}
            >
                Cancel
            </Button>
            <Button
                type="submit"
                variant="primary"
                isLoading={isPending}
                data-cy="data-cy-sign-loan-button"
            >
                Sign
            </Button>
        </form>
    </Box>);
}
