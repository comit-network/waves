import { Center, Text, VStack } from "@chakra-ui/react";
import * as React from "react";
import QRCode from "react-qr-code";
import { useAddress } from "../walletHooks";

export default function AddressQr() {
    const { data: address, error, isPending } = useAddress();

    if (isPending) {
        return <span>{"Loading..."}</span>;
    }
    if (error) {
        return <span>{`Something went wrong: ${error.message}`}</span>;
    }

    return (<Center
        bg="gray.100"
        h="10em"
        color="white"
        borderRadius={"md"}
    >
        <VStack>
            <Text textStyle="lgGray">Address</Text>
            <QRCode value={address!} size={100} />
            <Text
                textStyle="mdGray"
                maxWidth={"15em"}
                isTruncated
                data-cy="data-cy-wallet-address-text-field"
            >
                {address!}
            </Text>
        </VStack>
    </Center>);
}
