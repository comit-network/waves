import { Box, ChakraProvider, Heading } from "@chakra-ui/react";
import * as React from "react";
import { useAsync } from "react-async";
import { cancelLoan, getLoanToSign, getWalletBalance, getWalletStatus } from "./background-proxy";
import AddressQr from "./components/AddressQr";
import WalletBalances from "./components/Balances";
import ConfirmLoan from "./components/ConfirmLoan";
import CreateOrUnlockWallet from "./components/CreateOrUnlockWallet";
import { Status } from "./models";
import theme from "./theme";

const App = () => {
    const walletStatusHook = useAsync({ promiseFn: getWalletStatus });
    const walletBalanceHook = useAsync({ promiseFn: getWalletBalance });
    const loanToSignHook = useAsync({ promiseFn: getLoanToSign });

    let { data: walletStatus, reload: reloadWalletStatus } = walletStatusHook;
    let { data: balanceUpdates, reload: reloadWalletBalances } = walletBalanceHook;
    let { data: loanToSign, reload: reloadLoanToSing } = loanToSignHook;

    const refreshAll = async () => {
        await reloadWalletBalances();
        await reloadWalletStatus();
        await reloadLoanToSing();
    };

    return (
        <ChakraProvider theme={theme}>
            <Box h={600} w={400}>
                {walletStatus?.status === Status.Loaded
                    && <>
                        {balanceUpdates && <WalletBalances balanceUpdates={balanceUpdates} />}
                        <AddressQr />
                        {loanToSign
                            && <ConfirmLoan
                                onCancel={async () => {
                                    await cancelLoan(loanToSign!);
                                    await refreshAll();
                                }}
                                onSuccess={refreshAll}
                                loanToSign={loanToSign}
                            />}
                    </>}
                {walletStatus?.status === Status.NotLoaded
                    && <>
                        <Heading>Unlock Wallet</Heading>
                        <CreateOrUnlockWallet
                            onUnlock={refreshAll}
                            status={Status.NotLoaded}
                        />
                    </>}

                {walletStatus?.status === Status.None
                    && <>
                        <Heading>Create Wallet</Heading>
                        <CreateOrUnlockWallet
                            onUnlock={refreshAll}
                            status={Status.None}
                        />
                    </>}
            </Box>
        </ChakraProvider>
    );
};

export default App;
