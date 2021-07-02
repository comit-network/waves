import { Box, HStack, Image, Text } from "@chakra-ui/react";
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
    let elements = balanceUpdates.map((be) => balanceEntry(be));

    return (
        <HStack justify="center">
            {elements}
        </HStack>
    );
}

export default WalletBalances;
