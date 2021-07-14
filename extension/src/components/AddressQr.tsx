import { Center, Text, VStack } from "@chakra-ui/react";
import * as React from "react";
import { Async } from "react-async";
import QRCode from "react-qr-code";
import { getAddress } from "../background-proxy";

export default function AddressQr() {
    return (<Center
        bg="gray.100"
        h="10em"
        color="white"
        borderRadius={"md"}
    >
        <Async promiseFn={getAddress}>
            {({ data, error, isPending }) => {
                if (isPending) return "Loading...";
                if (error) return `Something went wrong: ${error.message}`;
                if (data) {
                    return (
                        <VStack>
                            <Text textStyle="lgGray">Address</Text>
                            <QRCode value={data} size={100} />
                            <Text
                                textStyle="mdGray"
                                maxWidth={"15em"}
                                isTruncated
                                data-cy="data-cy-wallet-address-text-field"
                            >
                                {data}
                            </Text>
                        </VStack>
                    );
                }
                return null;
            }}
        </Async>
    </Center>);
}
