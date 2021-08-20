import { Box, Center, Flex, Grid, GridItem, Image, Spacer, Text } from "@chakra-ui/react";
import React from "react";
import { BTC_TICKER, TradeSide, USDT_TICKER } from "../background/api";
import Bitcoin from "./bitcoin.svg";
import Usdt from "./tether.svg";

interface YouSwapItemProps {
    tradeSide: TradeSide;
    action: "send" | "receive";
}

export default function YouSwapItem({
    tradeSide: {
        ticker,
        amount,
        balanceBefore,
        balanceAfter,
    },
    action,
}: YouSwapItemProps) {
    return (
        <Box w="100%">
            <Flex>
                <Box h="40px" p="1">
                    <Text>You {action}</Text>
                </Box>
                <Spacer />
                <Box w="40px" h="40px">
                    {ticker === BTC_TICKER && <Image src={Bitcoin} h="32px" />}
                    {ticker === USDT_TICKER && <Image src={Usdt} h="32px" />}
                </Box>
                <Box h="40px" justify="right" p="1">
                    <Text align="center" justify="center">
                        {ticker}
                    </Text>
                </Box>
            </Flex>
            <Box bg="gray.100" borderRadius={"md"}>
                <Center>
                    <Box align="center" w="90%">
                        <Grid
                            templateRows="repeat(2, 2fr)"
                            templateColumns="repeat(2, 1fr)"
                        >
                            <GridItem colSpan={1}>
                                <Text align="left" textStyle="smGray">
                                    Old balance
                                </Text>
                            </GridItem>
                            <GridItem colSpan={1}>
                                <Text align="right" textStyle="smGray">
                                    {balanceBefore}
                                </Text>
                            </GridItem>
                            <GridItem colSpan={4}>
                                <Text align="center" textStyle="lg">
                                    {amount}
                                </Text>
                            </GridItem>
                            <GridItem colSpan={1}>
                                <Text align="left" textStyle="smGray">
                                    New balance
                                </Text>
                            </GridItem>
                            <GridItem colSpan={1}>
                                <Text align="right" textStyle="smGray">
                                    {balanceAfter}
                                </Text>
                            </GridItem>
                        </Grid>
                    </Box>
                </Center>
            </Box>
        </Box>
    );
}
