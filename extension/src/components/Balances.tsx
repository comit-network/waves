import { Box, HStack, Image, Text } from "@chakra-ui/react";
import { faPoo } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { BalanceEntry, BTC_TICKER, USDT_TICKER } from "../background/api";
import Btc from "./bitcoin.svg";
import Usdt from "./tether.svg";

interface BalancesProps {
    balanceUpdates: BalanceEntry[];
}

function balanceEntry(balance: BalanceEntry) {
    let image;
    if (balance.ticker === USDT_TICKER) {
        image = (<Box w="20px" h="20px">
            <Image src={Usdt} h="20px" />
        </Box>);
    } else if (balance.ticker === BTC_TICKER) {
        image = (<Box w="20px" h="20px">
            <Image src={Btc} h="20px" />
        </Box>);
    } else {
        image = (<Box w="20px" h="20px">
            <FontAwesomeIcon height="20px" icon={faPoo} />
        </Box>);
    }

    return <Box key={balance.ticker}>
        <HStack>
            {image}
            <HStack>
                <Text textStyle="smGray">
                    {balance.ticker}:
                </Text>
                <Text textStyle="smGray" data-cy={`data-cy-${balance.ticker}-balance-text-field`}>
                    {balance.value}
                </Text>
            </HStack>
        </HStack>
    </Box>;
}

function WalletBalances({ balanceUpdates }: BalancesProps) {
    let elements = balanceUpdates
        .sort((a, b) => a.ticker.localeCompare(b.ticker))
        .map((be) => balanceEntry(be));

    return (
        <HStack justify="center">
            {elements}
        </HStack>
    );
}

export default WalletBalances;
