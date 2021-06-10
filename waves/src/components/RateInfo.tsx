import { Box, Text } from "@chakra-ui/react";
import React from "react";
import { Rate } from "../App";

interface RateInfoProps {
    rate: Rate;
    direction: "ask" | "bid";
}

function RateInfo({ rate, direction }: RateInfoProps) {
    switch (direction) {
        case "ask":
            return <Box>
                <Text textStyle="smGray">{rate.ask} USDT ~ 1 BTC</Text>
            </Box>;
        case "bid":
            return <Box>
                <Text textStyle="smGray">1 BTC ~ {rate.bid} USDT</Text>
            </Box>;
    }
}

export default RateInfo;