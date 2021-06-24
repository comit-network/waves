import { Box, HStack, Image, Text, VStack } from "@chakra-ui/react";
import React from "react";
import { BalanceEntry, BalanceUpdate, BTC_TICKER, USDT_TICKER } from "../models";
import Btc from "./bitcoin.svg";
import Usdt from "./tether.svg";

interface BalancesProps {
    balanceUpdates: BalanceUpdate;
}

function balanceEntry(balance: BalanceEntry) {
    let image;
    if (balance.ticker === USDT_TICKER) {
        image = <Image src={Usdt} h="20px" />;
    } else if (balance.ticker === BTC_TICKER) {
        image = <Image src={Btc} h="20px" />;
    }
    return <Box key={balance.ticker}>
        <HStack>
            <Box>
                {image}
            </Box>
            <Box>
                <Text textStyle="smGray" data-cy={`usdt-${balance.ticker}`}>{balance.ticker}: {balance.value}</Text>
            </Box>
        </HStack>
    </Box>;
}

function WalletBalances({ balanceUpdates }: BalancesProps) {
    let elements = balanceUpdates.balances.map((be) => balanceEntry(be));

    return (
        <VStack align="center">
            {elements}
        </VStack>
    );
}

export default WalletBalances;
