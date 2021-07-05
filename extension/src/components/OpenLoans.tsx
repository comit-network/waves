import { Box, Button, FormControl, FormErrorMessage, HStack, Image, VStack } from "@chakra-ui/react";
import { Accordion, AccordionButton, AccordionIcon, AccordionItem, AccordionPanel } from "@chakra-ui/react";
import Debug from "debug";
import moment from "moment";
import * as React from "react";
import { useAsync } from "react-async";
import { repayLoan } from "../background-proxy";
import { LoanDetails } from "../models";
import Btc from "./bitcoin.svg";
import Usdt from "./tether.svg";

const error = Debug("openloans:error");

interface OpenLoansProps {
    openLoans: LoanDetails[] | undefined;
    onRepayed: () => Promise<void>;
}

export default function OpenLoans({ openLoans, onRepayed }: OpenLoansProps) {
    return (<Accordion allowMultiple>
        {openLoans && openLoans.sort((a, b) => a.term - b.term)
            .map(function(loanDetails, index) {
                return <OpenLoan
                    key={loanDetails.txId}
                    loanDetails={loanDetails}
                    onRepayed={onRepayed}
                    index={index}
                />;
            })}
    </Accordion>);
}

interface OpenLoanProps {
    loanDetails: LoanDetails;
    onRepayed: () => Promise<void>;
    index: number;
}

function OpenLoan({ loanDetails, onRepayed, index }: OpenLoanProps) {
    let { isLoading: isRepaying, isRejected: repayFailed, run: repay } = useAsync({
        deferFn: async ([txid]) => {
            await repayLoan(txid);
            await onRepayed();
        },
        onReject: (e) => error("Failed to repay loan: %s", e),
    });

    // TODO: get current block height from explorer
    const currentHeight = 10;
    // this will format the time nicely into something like : `in 13 hours` or `in 1 month`.
    const deadline = moment().add((currentHeight - loanDetails.term) * 60, "hours").fromNow();

    return <AccordionItem>
        <h2>
            <AccordionButton>
                <HStack spacing="24px">
                    <Box>
                        {"#"}
                        {index + 1}:
                    </Box>
                    <Box>
                        {loanDetails.principalRepayment}
                        {" "}
                        {loanDetails.principal.ticker}
                    </Box>
                    <Box w="32px" h="32px">
                        <Image src={Usdt} h="32px" />
                    </Box>
                    <Box>are due {deadline}</Box>
                </HStack>
                <AccordionIcon />
            </AccordionButton>
        </h2>
        <AccordionPanel pb={4}>
            <form
                onSubmit={e => {
                    e.preventDefault();
                    repay(loanDetails.txId);
                }}
            >
                <VStack>
                    <HStack>
                        <Box>
                            Principal amount: {loanDetails.principal.amount}
                        </Box>
                        <Box w="32px" h="32px">
                            <Image src={Usdt} h="32px" />
                        </Box>
                    </HStack>
                    <HStack>
                        <Box>
                            Collateral amount: {loanDetails.collateral.amount}
                        </Box>
                        <Box w="32px" h="32px">
                            <Image src={Btc} h="32px" />
                        </Box>
                    </HStack>
                    <HStack>
                        <Box>
                            Repayment amount: {loanDetails.principalRepayment}
                        </Box>
                        <Box w="32px" h="32px">
                            <Image src={Usdt} h="32px" />
                        </Box>
                    </HStack>
                    <Box>
                        Loan term: {loanDetails.term}
                    </Box>
                    <FormControl id="repayment" isInvalid={repayFailed}>
                        <Box>
                            <Button type="submit" isLoading={isRepaying}>
                                Repay
                            </Button>
                            <FormErrorMessage>Failed to repay loan.</FormErrorMessage>
                        </Box>
                    </FormControl>
                </VStack>
            </form>
        </AccordionPanel>
    </AccordionItem>;
}
