import { Box, Center, ChakraProvider, Heading } from "@chakra-ui/react";
import { faBug } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import * as React from "react";
import { useAsync } from "react-async";
import {
    getBalances,
    getLoanToSign,
    getOpenLoans,
    getSwapToSign,
    getWalletStatus,
    rejectLoan,
    rejectSwap,
} from "./background-proxy";
import AddressQr from "./components/AddressQr";
import WalletBalances from "./components/Balances";
import ConfirmLoan from "./components/ConfirmLoan";
import ConfirmSwap from "./components/ConfirmSwap";
import CreateOrUnlockWallet from "./components/CreateOrUnlockWallet";
import OpenLoans from "./components/OpenLoans";
import WithdrawAll from "./components/WithdrawAll";
import { Status } from "./models";
import theme from "./theme";

const App = () => {
    const walletStatusHook = useAsync({ promiseFn: getWalletStatus });
    const walletBalanceHook = useAsync({ promiseFn: getBalances });
    const swapToSignHook = useAsync({ promiseFn: getSwapToSign });
    const loanToSignHook = useAsync({ promiseFn: getLoanToSign });
    const openLoansHook = useAsync({ promiseFn: getOpenLoans });

    let { data: walletStatus, reload: reloadWalletStatus } = walletStatusHook;
    let { data: balanceUpdates, reload: reloadWalletBalances } = walletBalanceHook;
    let { data: swapToSign, reload: reloadSwapToSign } = swapToSignHook;
    let { data: loanToSign, reload: reloadLoanToSign } = loanToSignHook;
    let { data: openLoans, reload: reloadOpenLoans } = openLoansHook;

    const refreshAll = async () => {
        await reloadWalletBalances();
        await reloadWalletStatus();
        await reloadSwapToSign();
        await reloadLoanToSign();
        await reloadOpenLoans();
    };

    // we want to either sign a swap or the loan but not both:
    let signLoan = false;
    if (!swapToSign && loanToSign) {
        signLoan = true;
    }

    return (
        <ChakraProvider theme={theme}>
            <Box h={600} w={400}>
                {walletStatus?.status === Status.Loaded
                    && <>
                        {balanceUpdates && <WalletBalances balanceUpdates={balanceUpdates} />}
                        {!signLoan && !swapToSign && <AddressQr />}
                        {!signLoan && !swapToSign && <WithdrawAll />}
                        {!signLoan && !swapToSign && <OpenLoans openLoans={openLoans} onRepayed={refreshAll} />}

                        {swapToSign && <ConfirmSwap
                            onCancel={async (tabId: number) => {
                                await rejectSwap(tabId);
                                await refreshAll();
                            }}
                            onSuccess={refreshAll}
                            swapToSign={swapToSign!}
                        />}
                        {signLoan
                            && <ConfirmLoan
                                onCancel={async (tabId: number) => {
                                    await rejectLoan(tabId);
                                    await refreshAll();
                                }}
                                onSuccess={refreshAll}
                                loanToSign={loanToSign!}
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
                {!walletStatus
                    && <Center>
                        Something is wrong. Can you catch the <FontAwesomeIcon size="7x" icon={faBug} />?
                    </Center>}
            </Box>
        </ChakraProvider>
    );
};

export default App;
