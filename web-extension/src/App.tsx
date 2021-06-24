import { Box, ChakraProvider, Heading } from "@chakra-ui/react";
import * as React from "react";
import { useAsync } from "react-async";
import { getWalletBalance, getWalletStatus } from "./background-proxy";
import AddressQr from "./components/AddressQr";
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
                        {balanceUpdates && <WalletBalances balanceUpdates={balanceUpdates} />}
                        <AddressQr />
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

export default App;
