import { Box, Center, ChakraProvider, Heading, Text, VStack } from "@chakra-ui/react";
import * as React from "react";
import { Async, useAsync } from "react-async";
import QRCode from "react-qr-code";
import { getAddress, getWalletBalance, getWalletStatus } from "./background-proxy";
import WalletBalances from "./components/Balances";
import CreateOrUnlockWallet from "./components/CreateOrUnlockWallet";
import { Status } from "./models";
import theme from "./theme";

const App = () => {
    const walletStatusHook = useAsync({ promiseFn: getWalletStatus });
    const walletBalanceHook = useAsync({ promiseFn: getWalletBalance });

    let { data: walletStatus, reload: reloadWalletStatus } = walletStatusHook;
    let { data: balanceUpdates, reload: reloadWalletBalances } = walletBalanceHook;

    return (
        <ChakraProvider theme={theme}>
            <Box h={600} w={400}>
                {walletStatus?.status === Status.Loaded
                    && <>
                        <AddressQr />
                        {balanceUpdates && <WalletBalances balanceUpdates={balanceUpdates} />}
                    </>}
                {walletStatus?.status === Status.NotLoaded
                    && <>
                        <Heading>Unlock Wallet</Heading>
                        <CreateOrUnlockWallet
                            onUnlock={async () => {
                                await reloadWalletBalances();
                                await reloadWalletStatus();
                            }}
                            status={Status.NotLoaded}
                        />
                    </>}

                {walletStatus?.status === Status.None
                    && <>
                        <Heading>Create Wallet</Heading>
                        <CreateOrUnlockWallet
                            onUnlock={async () => {
                                await reloadWalletBalances();
                                await reloadWalletStatus();
                            }}
                            status={Status.None}
                        />
                    </>}
            </Box>
        </ChakraProvider>
    );
};

const AddressQr = () => (
    <Center
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
                            <Text textStyle="mdGray" maxWidth={"15em"} isTruncated data-cy="wallet-address-textfield">
                                {data}
                            </Text>
                        </VStack>
                    );
                }
                return null;
            }}
        </Async>
    </Center>
);

export default App;
